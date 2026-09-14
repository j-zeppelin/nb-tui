use notify::{EventKind, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{io, process::Command};
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
    fn into_nb_result(self, args: impl Into<String>) -> Result<(), NbError>;
}

impl OutputExt for std::process::Output {
    fn into_nb_result(self, args: impl Into<String>) -> Result<(), NbError> {
        self.status
            .success()
            .then_some(())
            .ok_or_else(|| NbError::NbFailure {
                args: args.into(),
                stderr: String::from_utf8_lossy(&strip_ansi_escapes::strip(&self.stderr))
                    .into_owned(),
            })
    }
}

pub fn remove_item(id: &NbItemId) -> Result<(), NbError> {
    Command::new("nb")
        .args(["rm", &id.to_string(), "--force", "--no-color"])
        .output()
        .map_err(NbError::ExecutionFailure)?
        .into_nb_result(format!("rm {id}"))
}

pub fn check_nb_available() -> Result<(), NbError> {
    Command::new("nb")
        .arg("--version")
        .output()
        .map_err(NbError::ExecutionFailure)?;
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
