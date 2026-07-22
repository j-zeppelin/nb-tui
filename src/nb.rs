use notify::Watcher;
use std::collections::HashSet;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{io, process::Command};
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

pub fn check_nb_available() -> Result<(), NbError> {
    Command::new("nb")
        .arg("--version")
        .output()
        .map_err(NbError::ExecutionFailure)?;
    Ok(())
}

// nb root handling

pub enum NbRoot {
    Local(PathBuf),
    Global(PathBuf),
}

impl NbRoot {
    pub fn resolve(explicit: Option<PathBuf>) -> io::Result<Self> {
        match explicit {
            Some(path) => Ok(Self::Local(path)),
            None => Ok(Self::Global(global_nb_dir()?)),
        }
    }

    pub fn active_notebook_dir(&self) -> PathBuf {
        match self {
            NbRoot::Local(path) => path.clone(),
            NbRoot::Global(root) => root.join(get_current_notebook(root)),
        }
    }

    pub fn watcher_root(&self) -> &Path {
        match self {
            NbRoot::Local(path) => path,
            NbRoot::Global(root) => root,
        }
    }
}

fn global_nb_dir() -> io::Result<PathBuf> {
    std::env::home_dir()
        .map(|home| home.join(".nb"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not find HOME directory"))
}

// folder navigation

pub struct FolderNav {
    base: PathBuf,
    stack: Vec<PathBuf>,
}

impl FolderNav {
    pub fn new(base: PathBuf) -> Self {
        Self {
            base,
            stack: Vec::new(),
        }
    }

    pub fn reset(&mut self, new_base: PathBuf) {
        self.base = new_base;
        self.stack.clear();
    }

    pub fn current_dir(&self) -> PathBuf {
        match self.stack.last() {
            Some(dir) => dir.clone(),
            None => self.base.clone(),
        }
    }

    pub fn enter(&mut self, folder_name: &str) {
        let next = self.current_dir().join(folder_name);
        self.stack.push(next);
    }

    pub fn go_back(&mut self) -> bool {
        self.stack.pop().is_some()
    }
}

// fs watcher

pub enum FsEvent {
    Changed,
}

pub fn spawn_fs_watcher(
    nb_root: &Path,
    tx: Sender<FsEvent>,
) -> notify::Result<notify::RecommendedWatcher> {
    let mut last_sent: Option<Instant> = None;
    const DEBOUNCE: Duration = Duration::from_millis(150);

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let Ok(event) = res else { return };
        if event.paths.iter().any(|p| is_ignored(p)) {
            return;
        }
        let now = Instant::now();
        if last_sent.map_or(true, |t| now.duration_since(t) > DEBOUNCE) {
            last_sent = Some(now);
            let _ = tx.send(FsEvent::Changed);
        }
    })?;

    watcher.watch(nb_root, notify::RecursiveMode::Recursive)?;
    Ok(watcher)
}

// notebook and note handling

pub fn get_notebooks(nb_root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(nb_root) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect()
}

pub fn scan_folder(dir: &Path) -> io::Result<Vec<NbItem>> {
    let pinned = read_pindex(dir);

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| !is_ignored(&e.path()))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    Ok(entries
        .into_iter()
        .enumerate()
        .filter_map(|(i, entry)| NbItem::parse(i, entry, &pinned))
        .collect())
}

pub fn get_current_notebook(nb_root: &Path) -> String {
    let path = nb_root.join(".current");

    match fs::read_to_string(path) {
        Ok(contents) => contents
            .lines()
            .next()
            .unwrap_or(&get_first_notebook(nb_root))
            .to_string(),
        Err(_) => get_first_notebook(nb_root),
    }
}

fn get_first_notebook(nb_root: &Path) -> String {
    // home is the default notebook name of `nb`, hence we use "home"
    // as the fallback
    match fs::read_dir(nb_root) {
        Ok(mut e) => match e.next() {
            Some(Ok(entry)) => entry.file_name().to_string_lossy().into_owned(),
            _ => "home".to_string(),
        },

        Err(_) => "home".to_string(),
    }
}

fn read_pindex(dir: &Path) -> HashSet<String> {
    let path = dir.join(".pindex");
    match fs::read_to_string(path) {
        Ok(contents) => contents
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect(),
        Err(_) => HashSet::new(),
    }
}

fn is_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with(".git") || n.starts_with(".cache"))
        .unwrap_or(false)
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

impl NbItemKind {
    pub fn from_ext(ext: &str) -> Self {
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

            let kind = NbItemKind::from_ext(ext);

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
