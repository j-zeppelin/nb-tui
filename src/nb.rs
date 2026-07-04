use std::sync::mpsc::Sender;
use std::thread;
use std::{ffi::OsStr, io, process::Command};

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use thiserror::Error;

const INDICATOR_ENV: &[(&str, &str)] = &[
    ("NB_INDICATOR_AUDIO", "🔉"),
    ("NB_INDICATOR_BOOKMARK", "🔖"),
    ("NB_INDICATOR_DOCUMENT", "📄"),
    ("NB_INDICATOR_EBOOK", "📖"),
    ("NB_INDICATOR_ENCRYPTED", "🔒"),
    ("NB_INDICATOR_FOLDER", "📂"),
    ("NB_INDICATOR_IMAGE", "🌄"),
    ("NB_INDICATOR_PINNED", "📌"),
    ("NB_INDICATOR_TODO", "✔️ "),
    ("NB_INDICATOR_TODO_DONE", "✅"),
    ("NB_INDICATOR_VIDEO", "📹"),
];

#[derive(Error, Debug)]
pub enum NbError {
    /// Error type for when `nb` cannot be executed for any reason
    #[error("could not execute nb: {0}")]
    ExecutionFailure(#[from] io::Error),

    /// Error type for when `nb` itself fails
    #[error("nb {args}: {stderr}")]
    NbFailure { args: String, stderr: String },
}

/// check if `nb` can be executed, otherwise return an [NbError]
pub fn check_nb_available() -> Result<(), NbError> {
    Command::new("nb")
        .arg("--version")
        .output()
        .map_err(NbError::ExecutionFailure)?;
    Ok(())
}

/// execute `nb` with given args, `--no-color` is always passed to `nb`
/// expects nb to return valid UTF-8
pub fn execute<I, S>(args: I) -> Result<String, NbError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut args: Vec<_> = args
        .into_iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect();

    args.push("--no-color".to_string());

    let output = Command::new("nb")
        .args(&args)
        .envs(INDICATOR_ENV.iter().copied())
        .output()?;

    if !output.status.success() {
        return Err(NbError::NbFailure {
            args: args.join(" "),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// executes `nb` in a background thread and sends back the result tagged with `tag`
/// wraps [execute]
pub fn execute_async<I, S, T>(args: I, tag: T, tx: Sender<(T, Result<String, NbError>)>)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    T: Send + 'static,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect();

    thread::spawn(move || {
        let result = execute(args);
        let _ = tx.send((tag, result));
    });
}

/// helper function for opening notebook files with the default editor
pub fn open_in_editor(term: &mut DefaultTerminal, id: usize) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;

    Command::new("nb")
        .args(["edit", &id.to_string()])
        .status()?;

    enable_raw_mode()?;

    execute!(term.backend_mut(), EnterAlternateScreen)?;
    term.clear()?;
    Ok(())
}

#[derive(Debug, PartialEq, Clone)]
pub enum NbItemKind {
    Note { preview: Option<String> },
    Bookmark { url: Option<String> },
    Todo { done: bool },
    Image,
    Audio,
    Video,
    Document,
    Ebook,
    Folder,
}

#[derive(Debug, Clone)]
pub struct NbItem {
    pub id: usize,
    pub title: String,
    pub kind: NbItemKind,
    pub pinned: bool,
    pub encrypted: bool,
}

impl NbItem {
    pub fn parse(line: &str) -> Option<Self> {
        // parse id
        let line = line.trim_start();
        let rest = line.strip_prefix('[')?;
        let close_idx = rest.find(']')?;
        let id = rest[..close_idx].trim().parse::<usize>().ok()?;

        // parse pinned
        let rest = rest[close_idx + 1..].trim_start();
        let mut pinned = false;
        let rest = if let Some(stripped) = rest.strip_prefix('📌') {
            pinned = true;
            stripped.trim()
        } else {
            rest.trim()
        };

        // parse encrypted (pinned always comes first)
        let mut encrypted = false;
        let rest = if let Some(stripped) = rest.strip_prefix('🔒') {
            encrypted = true;
            stripped.trim()
        } else {
            rest.trim()
        };

        // parse kind and title
        let (kind, title) = if let Some(rest) = rest.strip_prefix('📂') {
            (NbItemKind::Folder, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('📄') {
            (NbItemKind::Document, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('🌄') {
            (NbItemKind::Image, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('📹') {
            (NbItemKind::Video, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('📖') {
            (NbItemKind::Ebook, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('🔉') {
            (NbItemKind::Audio, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('✅') {
            (NbItemKind::Todo { done: true }, rest.trim())
        } else if let Some(rest) = rest.strip_prefix("✔️ ") {
            (NbItemKind::Todo { done: false }, rest.trim())
        } else if let Some(rest) = rest.strip_prefix('🔖') {
            if encrypted {
                (NbItemKind::Bookmark { url: None }, rest.trim())
            } else {
                let idx = rest.rfind('(')?;
                let name = rest[..idx].trim();
                let url = rest[idx + 1..].trim_end_matches(')').trim();
                (
                    NbItemKind::Bookmark {
                        url: Some(url.to_string()),
                    },
                    name,
                )
            }
        } else {
            if let Some((title, preview)) = rest.split_once('·') {
                (
                    NbItemKind::Note {
                        preview: Some(preview.trim().trim_matches('\"').to_string()),
                    },
                    title.trim(),
                )
            } else {
                (NbItemKind::Note { preview: None }, rest.trim())
            }
        };

        Some(Self {
            id,
            title: title.to_string(),
            kind,
            pinned,
            encrypted,
        })
    }
}

mod tests {
    use super::*;

    #[test]
    fn note_parses_correctly() {
        let item = NbItem::parse("[1] note.md · \"this is a note\"").unwrap();

        assert_eq!(item.id, 1);
        assert_eq!(item.title, "note.md");
        assert_eq!(
            item.kind,
            NbItemKind::Note {
                preview: Some("this is a note".to_string())
            }
        );
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn note_with_title_parses_correctly() {
        let item = NbItem::parse("[2] note with title").unwrap();

        assert_eq!(item.id, 2);
        assert_eq!(item.title, "note with title");
        assert_eq!(item.kind, NbItemKind::Note { preview: None });
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn pinned_note_with_title_parses_correctly() {
        let item = NbItem::parse("[3] 📌 note with title").unwrap();

        assert_eq!(item.id, 3);
        assert_eq!(item.title, "note with title");
        assert_eq!(item.kind, NbItemKind::Note { preview: None });
        assert_eq!(item.pinned, true);
    }

    #[test]
    fn folder_parses_correctly() {
        let item = NbItem::parse("[4] 📂 folder").unwrap();

        assert_eq!(item.id, 4);
        assert_eq!(item.title, "folder");
        assert_eq!(item.kind, NbItemKind::Folder);
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn todo_parses_correctly() {
        let item = NbItem::parse("[5] ✔️  [ ] todo").unwrap();

        assert_eq!(item.id, 5);
        assert_eq!(item.title, "[ ] todo");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn finished_todo_parses_correctly() {
        let item = NbItem::parse("[6] ✅ [x] todo2").unwrap();

        assert_eq!(item.id, 6);
        assert_eq!(item.title, "[x] todo2");
        assert_eq!(item.kind, NbItemKind::Todo { done: true });
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn bookmark_parses_correctly() {
        let item = NbItem::parse("[7] 🔖 Google (www.google.com)").unwrap();

        assert_eq!(item.id, 7);
        assert_eq!(item.title, "Google");
        assert_eq!(
            item.kind,
            NbItemKind::Bookmark {
                url: Some("www.google.com".to_string())
            }
        );
        assert_eq!(item.pinned, false);
    }
}
