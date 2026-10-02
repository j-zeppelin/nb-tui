use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify::{EventKind, Watcher};
use ratatui::DefaultTerminal;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{io, process::Command};
use std::{path::Path, process::Stdio};
use thiserror::Error;

use crate::nb::item::NbItemId;

pub mod item;
pub mod root;

// nb calls

#[derive(Error, Debug)]
pub enum NbError {
    /// Error type for when `nb` cannot be executed for any reason
    #[error("could not execute nb: {0}")]
    ExecutionFailure(#[from] io::Error),

    /// Error type for when `nb` itself fails
    #[error("nb {args}: {stderr}")]
    NbFailure { args: String, stderr: String },
}

trait OutputExt {
    fn into_nb_result(self, args: impl Into<String>) -> Result<String, NbError>;
}

impl OutputExt for std::process::Output {
    fn into_nb_result(self, args: impl Into<String>) -> Result<String, NbError> {
        self.status
            .success()
            .then_some(
                String::from_utf8_lossy(&strip_ansi_escapes::strip(&self.stdout)).into_owned(),
            )
            .ok_or_else(|| NbError::NbFailure {
                args: args.into(),
                stderr: String::from_utf8_lossy(&strip_ansi_escapes::strip(&self.stderr))
                    .into_owned(),
            })
    }
}

pub fn add_item(filename: &str, pinned: bool) -> Result<(), NbError> {
    let output = Command::new("nb")
        .stdin(Stdio::null())
        .args(["add", "--title", filename, "--no-color"])
        .output()
        .map_err(NbError::ExecutionFailure)?
        .into_nb_result(format!("add --title {filename}"))?;

    if pinned {
        let Some(id) = get_id_from_nb_output(&output) else {
            return Err(NbError::NbFailure {
                args: "add --title {filename}".to_string(),
                stderr: "could not parse id from nb".to_string(),
            });
        };

        return pin_item(id);
    }
    Ok(())
}

pub fn remove_item(id: &NbItemId) -> Result<(), NbError> {
    let _ = Command::new("nb")
        .args(["rm", &id.to_string(), "--force", "--no-color"])
        .output()
        .map_err(NbError::ExecutionFailure)?
        .into_nb_result(format!("rm {id}"))?;

    Ok(())
}

pub fn pin_item(id: NbItemId) -> Result<(), NbError> {
    let _ = Command::new("nb")
        .args(["pin", &id.to_string()])
        .output()
        .map_err(NbError::ExecutionFailure)?
        .into_nb_result(format!("pin {id}"))?;

    Ok(())
}

pub fn check_nb_available() -> Result<(), NbError> {
    Command::new("nb")
        .arg("--version")
        .status()
        .map_err(NbError::ExecutionFailure)?;
    Ok(())
}

pub fn open_in_editor(term: &mut DefaultTerminal, id: NbItemId) -> io::Result<()> {
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

// fs watcher
const IGNORED: &[&str; 4] = &[".git", ".cache", ".index", ".pindex"];

pub fn spawn_fs_watcher(
    nb_root: &Path,
    tx: Sender<EventKind>,
) -> notify::Result<notify::RecommendedWatcher> {
    const DEBOUNCE: Duration = Duration::from_millis(150);

    let mut last_sent: Option<Instant> = None;

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if event.paths.iter().any(|p| is_ignored(p)) {
            return;
        }
        let now = Instant::now();
        if last_sent.is_none_or(|t| now.duration_since(t) > DEBOUNCE) {
            last_sent = Some(now);
            let _ = tx.send(event.kind);
        }
    })?;

    watcher.watch(nb_root, notify::RecursiveMode::Recursive)?;
    Ok(watcher)
}

// notebook and note handling

pub fn is_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| IGNORED.contains(&n))
}

fn get_id_from_nb_output(output: &str) -> Option<NbItemId> {
    let start_idx = output.find(|c| c == '[')?;
    let end_idx = output[start_idx..].find(|c| c == ']')?;

    NbItemId::from_str(&output[start_idx + 1..start_idx + end_idx])
}

mod tests {
    use super::*;

    #[test]
    fn id_parses_correctly_no_folder() {
        let id = get_id_from_nb_output("Added: [10] amog[u]sss.md.md \"amogusss.md\"");

        assert_eq!(id, Some(NbItemId::new(None, 10)));
    }

    #[test]
    fn id_parses_correctly_with_folder() {
        let id = get_id_from_nb_output("Added: [deep/10] amog[u]sss.md.md \"amogusss.md\"");

        assert_eq!(id, Some(NbItemId::new(Some("deep".to_string()), 10)));
    }

    #[test]
    fn id_parses_incorrectly() {
        let id = get_id_from_nb_output("Added: [10 amogusss.md.md \"amogusss.md\"");

        // assert_eq!(id, None);
    }
}
