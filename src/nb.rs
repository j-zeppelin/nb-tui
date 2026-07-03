use std::{ffi::OsStr, io, process::Command};

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use thiserror::Error;

pub mod item;

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
