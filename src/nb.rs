use std::{ffi::OsStr, io, pin, process::Command};

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

pub fn execute_nb<I, S>(args: I) -> Result<String, NbError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = args
        .into_iter()
        .map(|a| a.as_ref().to_string_lossy().into_owned())
        .collect();

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

#[derive(Debug)]
pub struct NbNotebook {
    name: String,
    items: Vec<NbItem>,
}

impl NbNotebook {
    pub fn new(name: &str) -> Result<Self, NbError> {
        let output = execute_nb([
            &format!("{name}:ls"),
            "--no-header",
            "--no-footer",
            "--no-color",
            "-af",
        ])?;
        let items: Vec<_> = output.lines().map(|l| NbItem::parse(l).unwrap()).collect();

        Ok(Self {
            name: name.to_string(),
            items,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum NbItemKind {
    Note { preview: Option<String> },
    Bookmark { url: String },
    Todo { done: bool },
    Image,
    Audio,
    Video,
    Document,
    Ebook,
    Folder,
}

#[derive(Debug)]
pub struct NbItem {
    id: usize,
    title: String,
    kind: NbItemKind,
    pinned: bool,
}

impl NbItem {
    pub fn parse(line: &str) -> Option<Self> {
        // parse id
        let line = line.trim_start();
        let rest = line.strip_prefix('[')?;
        let close_idx = rest.find(']')?;
        let id = rest[..close_idx].trim().parse::<usize>().ok()?;

        // parse kind and pinned status
        let rest = rest[close_idx + 1..].trim_start();
        let mut pinned = false;

        let rest = if let Some(stripped) = rest.strip_prefix("📌") {
            pinned = true;
            stripped.trim()
        } else {
            rest.trim()
        };

        let (kind, title) = if let Some(rest) = rest.strip_prefix('🔖') {
            let idx = rest.rfind('(')?;
            let name = rest[..idx].trim();
            let url = rest[idx + 1..].trim_end_matches(')').trim();
            (
                NbItemKind::Bookmark {
                    url: url.to_string(),
                },
                name,
            )
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
        assert_eq!(item.title, "todo");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
        assert_eq!(item.pinned, false);
    }

    #[test]
    fn finished_todo_parses_correctly() {
        let item = NbItem::parse("[6] ✅ [x] todo2").unwrap();

        assert_eq!(item.id, 6);
        assert_eq!(item.title, "todo");
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
                url: "www.google.com".to_string()
            }
        );
        assert_eq!(item.pinned, false);
    }
}
