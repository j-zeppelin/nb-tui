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
    pub fn parse(id: usize, entry: DirEntry, pinned: &HashSet<String>) -> Option<Self> {
        let path = entry.path();
        let file_type = entry.file_type().ok()?;
        let filename = path.file_name()?.to_str()?;
        let extension = filename.rsplit_once('.').map(|(_, ext)| ext)?;

        if file_type.is_dir() {
            return Some(Self {
                id,
                title: filename.to_string(),
                kind: NbItemKind::Folder,
                pinned: pinned.contains(filename),
                encrypted: false,
            });
        }

        match extension {
            ".todo.md" => {
                todo!()
            }
            ".bookmark.md" => {
                todo!()
            }
            ".md" => {
                todo!()
            }
            _ => {
                todo!()
            }
        }
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

                name = buf.get(l_bracket_idx + 4..)?.to_string();
            }
        }

        Some((name, done))
    }
}

mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom, Write};
    use tempfile::tempfile;

    #[test]
    fn todo_parses_correctly() {
        let mut tmpfile: File = tempfile().unwrap();
        write!(tmpfile, "# [ ] todo").unwrap();
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        let result = NbItem::todo_info(tmpfile).unwrap();

        assert_eq!(result.0, "todo".to_string());
        assert_eq!(result.1, false);
    }
}
