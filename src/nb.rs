use std::collections::HashSet;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::{io, process::Command};

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use thiserror::Error;

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

pub struct NbClient {
    basepath: PathBuf,
}

impl NbClient {
    pub fn default() -> Self {
        Self {
            basepath: PathBuf::from("~/.nb"),
        }
    }

    /// execute `nb` with given args
    /// expects `nb` return valid UTF-8
    fn run(&self, args: &[&str]) -> Result<String, NbError> {
        let output = Command::new("nb").args(args).output()?;

        if !output.status.success() {
            return Err(NbError::NbFailure {
                args: args.join(" "),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

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
}

#[derive(Debug, PartialEq, Clone)]
pub enum NbItemKind {
    Note,
    Bookmark { url: String },
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
    pub fn parse(id: usize, entry: DirEntry, pinned: &HashSet<String>) -> Option<Self> {
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        let filename = path.file_name()?.to_str()?;

        if file_type.is_dir() {
            return Some(Self {
                id,
                title: filename.to_string(),
                kind: NbItemKind::Folder,
                pinned: pinned.contains(filename),
                encrypted: false,
            });
        }

        let encrypted = filename.ends_with(".enc");
        let pinned = pinned.contains(filename);
        let stripped_filename = filename.trim_end_matches(".enc");

        let kind_and_tile = if stripped_filename.ends_with(".todo.md") {
            let (title, done) = Self::todo_info(File::open(&path).ok()?)?;
            Some((title, NbItemKind::Todo { done }))
        } else if stripped_filename.ends_with(".bookmark.md") {
            let (title, url) = Self::bookmark_info(File::open(&path).ok()?)?;
            Some((title, NbItemKind::Bookmark { url }))
        } else {
            let ext = Path::new(stripped_filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let kind = kind_for_extension(ext);

            dbg!(filename);
            let title = match kind {
                NbItemKind::Note if ext == "md" || ext == "markdown" => {
                    Self::note_info(File::open(&path).ok()?).unwrap_or_else(|| filename.to_string())
                }
                _ => filename.to_string(),
            };

            Some((title, kind))
        };

        let (title, kind) = kind_and_tile?;

        Some(Self {
            id,
            title,
            kind,
            pinned,
            encrypted,
        })
    }

    fn todo_info(file: File) -> Option<(String, bool)> {
        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        let mut done = false;
        let mut name = String::new();

        loop {
            buf.clear();
            let bytes = reader.read_line(&mut buf).ok()?;
            if bytes == 0 {
                break;
            } // EOF

            if buf.trim_start().starts_with('#') {
                let l_bracket_idx = buf.find('[')?;

                if buf.get(l_bracket_idx + 2..=l_bracket_idx + 2)? == "]"
                    && buf.get(l_bracket_idx + 1..=l_bracket_idx + 1)? == "x"
                {
                    done = true;
                }

                name = buf.get(l_bracket_idx + 4..)?.trim().to_string();
            }
        }

        Some((name, done))
    }

    fn bookmark_info(file: File) -> Option<(String, String)> {
        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        let mut name = String::new();
        let mut url = String::new();

        loop {
            buf.clear();
            let bytes = reader.read_line(&mut buf).ok()?;
            if bytes == 0 {
                break;
            } // EOF

            if buf.trim_start().starts_with('#') {
                let end = buf.find('(')?;
                name = buf
                    .trim_start_matches('#')
                    .get(0..end - 2)?
                    .trim()
                    .to_string();
            } else if buf.trim_start().starts_with('<') {
                dbg!("help");
                let end = buf.find('>')?;

                url = buf.get(1..end)?.trim().to_string();
            }
        }

        Some((name, url))
    }

    fn note_info(file: File) -> Option<String> {
        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        let mut title = String::new();

        loop {
            buf.clear();
            let bytes = reader.read_line(&mut buf).ok()?;
            if bytes == 0 {
                break;
            } // EOF

            if buf.trim_start().starts_with('#') {
                title = buf.trim_start_matches('#').trim().to_string();
            }
        }

        if title.is_empty() { None } else { Some(title) }
    }
}

fn kind_for_extension(ext: &str) -> NbItemKind {
    match ext.to_ascii_lowercase().as_str() {
        // Text / note-like content
        "md" | "markdown" | "txt" | "text" | "rst" | "adoc" | "org" | "rs" | "js" | "ts"
        | "jsx" | "tsx" | "py" | "go" | "c" | "cpp" | "h" | "hpp" | "java" | "kt" | "rb"
        | "php" | "sh" | "bash" | "fish" | "zsh" | "toml" | "yaml" | "yml" | "json" | "xml"
        | "html" | "css" | "sql" | "lua" | "nix" | "vim" | "el" => NbItemKind::Note,

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "heic" => {
            NbItemKind::Image
        }

        // Audio
        "mp3" | "wav" | "flac" | "ogg" | "m4a" | "aac" | "opus" => NbItemKind::Audio,

        // Video
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "flv" | "wmv" => NbItemKind::Video,

        // Documents
        "pdf" | "doc" | "docx" | "odt" | "rtf" | "xls" | "xlsx" | "ppt" | "pptx" | "csv" => {
            NbItemKind::Document
        }

        // Ebooks
        "epub" | "mobi" | "azw" | "azw3" | "fb2" => NbItemKind::Ebook,

        _ => NbItemKind::Note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::tempdir;

    /// Writes `content` to `dir/name` and returns the resulting DirEntry,
    /// since DirEntry can't be constructed directly — it only comes from read_dir.
    fn write_and_get_entry(dir: &std::path::Path, name: &str, content: &str) -> DirEntry {
        fs::write(dir.join(name), content).unwrap();
        fs::read_dir(dir)
            .unwrap()
            .find_map(|e| {
                let e = e.ok()?;
                (e.file_name().to_str() == Some(name)).then_some(e)
            })
            .unwrap_or_else(|| panic!("could not find written file {name}"))
    }

    fn mkdir_and_get_entry(dir: &std::path::Path, name: &str) -> DirEntry {
        fs::create_dir(dir.join(name)).unwrap();
        fs::read_dir(dir)
            .unwrap()
            .find_map(|e| {
                let e = e.ok()?;
                (e.file_name().to_str() == Some(name)).then_some(e)
            })
            .unwrap_or_else(|| panic!("could not find created dir {name}"))
    }

    #[test]
    fn parses_folder() {
        let dir = tempdir().unwrap();
        let entry = mkdir_and_get_entry(dir.path(), "my-folder");

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "my-folder");
        assert_eq!(item.kind, NbItemKind::Folder);
        assert!(!item.pinned);
        assert!(!item.encrypted);
    }

    #[test]
    fn parses_pinned_folder() {
        let dir = tempdir().unwrap();
        let entry = mkdir_and_get_entry(dir.path(), "my-folder");
        let pinned: HashSet<String> = ["my-folder".to_string()].into_iter().collect();

        let item = NbItem::parse(0, entry, &pinned).unwrap();

        assert!(item.pinned);
    }

    #[test]
    fn parses_undone_todo() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(
            dir.path(),
            "20240101000001.todo.md",
            "# [ ] Water the plants\n",
        );

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "Water the plants");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
        assert!(!item.encrypted);
    }

    #[test]
    fn parses_done_todo() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(
            dir.path(),
            "20240101000001.todo.md",
            "# [x] Water the plants\n",
        );

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.kind, NbItemKind::Todo { done: true });
    }

    #[test]
    fn parses_bookmark() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(
            dir.path(),
            "20240101000001.bookmark.md",
            "# Rust Docs (doc.rust-lang.org)\n\n<https://doc.rust-lang.org>",
        );

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "Rust Docs");
        assert_eq!(
            item.kind,
            NbItemKind::Bookmark {
                url: "https://doc.rust-lang.org".to_string()
            }
        );
    }

    #[test]
    fn parses_plain_note() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(
            dir.path(),
            "20240101000001.md",
            "# Just a regular note\n\nSome body text.\n",
        );

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "Just a regular note");
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn note_without_heading_falls_back_to_filename() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(dir.path(), "20240101000001.md", "no heading here\n");

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "20240101000001.md");
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn parses_encrypted_todo() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(
            dir.path(),
            "20240101000001.todo.md.enc",
            "# [ ] Secret task\n",
        );

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert!(item.encrypted);
        assert_eq!(item.title, "Secret task");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
    }

    #[test]
    fn classifies_source_file_as_note_using_filename_as_title() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(dir.path(), "20240101000001.rs", "fn main() {}\n");

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        // .rs isn't "md"/"markdown" so title should stay as the filename,
        // not attempt heading extraction.
        assert_eq!(item.title, "20240101000001.rs");
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn classifies_image_extension() {
        let dir = tempdir().unwrap();
        // content doesn't need to be a real image for this parser, since
        // image kind never opens/reads the file
        let entry = write_and_get_entry(dir.path(), "20240101000001.png", "");

        let item = NbItem::parse(0, entry, &HashSet::new()).unwrap();

        assert_eq!(item.title, "20240101000001.png");
        assert_eq!(item.kind, NbItemKind::Image);
    }

    #[test]
    fn respects_pinned_set_for_files() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(dir.path(), "20240101000001.md", "# A note\n");
        let pinned: HashSet<String> = ["20240101000001.md".to_string()].into_iter().collect();

        let item = NbItem::parse(0, entry, &pinned).unwrap();

        assert!(item.pinned);
    }

    #[test]
    fn unpinned_file_not_in_pindex() {
        let dir = tempdir().unwrap();
        let entry = write_and_get_entry(dir.path(), "20240101000001.md", "# A note\n");
        let pinned: HashSet<String> = ["some-other-file.md".to_string()].into_iter().collect();

        let item = NbItem::parse(0, entry, &pinned).unwrap();

        assert!(!item.pinned);
    }
}
