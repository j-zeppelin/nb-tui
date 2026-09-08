use notify::{EventKind, Watcher};
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{io, process::Command};
use thiserror::Error;

const IGNORED: &[&str; 4] = &[".git", ".cache", ".index", ".pindex"];

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

pub fn remove_item(id: usize) -> Result<(), NbError> {
    let output = Command::new("nb")
        .arg("rm")
        .arg(id.to_string())
        .arg("--force")
        .output()
        .map_err(NbError::ExecutionFailure)?;

    if !output.status.success() {
        return Err(NbError::NbFailure {
            args: format!("rm {id}"),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }

    Ok(())
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

    pub fn global_root(&self) -> &Path {
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

    pub fn is_at_root(&self) -> bool {
        return self.stack.is_empty();
    }

    pub fn breadcrumbs(&self) -> Vec<String> {
        self.stack
            .iter()
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect()
    }
}

// fs watcher

pub fn spawn_fs_watcher(
    nb_root: &Path,
    tx: Sender<EventKind>,
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
            let _ = tx.send(event.kind);
        }
    })?;

    watcher.watch(nb_root, notify::RecursiveMode::Recursive)?;
    Ok(watcher)
}

// notebook and note handling

pub fn get_notebooks(nb_root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(nb_root) else {
        return vec!["home".to_string()];
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false) && !is_ignored(&e.path()))
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect()
}

pub fn scan_folder(dir: &Path) -> Vec<NbItem> {
    let index = read_index(dir);
    let pinned = read_pindex(dir);

    let mut items: Vec<_> = index
        .into_iter()
        .filter_map(|(i, entry)| NbItem::parse(i + 1, dir.join(entry), &pinned))
        .collect();

    items.sort_by_key(|item| {
        (
            cmp::Reverse(item.pinned),
            cmp::Reverse(matches!(item.kind, NbItemKind::Folder)),
            item.title.clone(),
        )
    });

    items
}

pub fn set_current_notebook(nb_root: &Path, notebook: &str) -> io::Result<()> {
    let path = nb_root.join(".current");
    fs::write(path, notebook)
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
        // maybe create .pindex manually in the future
        Err(_) => HashSet::new(),
    }
}

fn read_index(dir: &Path) -> HashMap<usize, String> {
    let path = dir.join(".index");
    match fs::read_to_string(path) {
        Ok(contents) => contents
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .enumerate()
            .map(|(i, l)| (i, l.to_string()))
            .collect(),
        // maybe create .index manually in the future
        Err(_) => HashMap::new(),
    }
}

fn is_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| IGNORED.contains(&n))
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
    pub filename: String,
    pub kind: NbItemKind,
    pub pinned: bool,
    pub encrypted: bool,
}

impl NbItem {
    pub fn parse(id: usize, entry: PathBuf, pinned: &HashSet<String>) -> Option<Self> {
        let path = entry.clone();
        let file_type = fs::metadata(entry).ok()?.file_type();
        let file_name = path.file_name()?.to_str()?;

        if file_type.is_dir() {
            return Some(Self {
                id,
                title: file_name.to_string(),
                filename: file_name.to_string(),
                kind: NbItemKind::Folder,
                pinned: pinned.contains(file_name),
                encrypted: false,
            });
        }

        let encrypted = file_name.ends_with(".enc");
        let pinned = pinned.contains(file_name);
        let stripped_filename = file_name.trim_end_matches(".enc");

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
                    Self::note_info(File::open(&path).ok()?)
                        .unwrap_or_else(|| file_name.to_string())
                }
                _ => file_name.to_string(),
            };

            Some((title, kind))
        };

        let (title, kind) = kind_and_tile?;

        Some(Self {
            id,
            title,
            filename: file_name.to_string(),
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
    use std::{collections::HashSet, io::Write};
    use tempfile::{Builder, NamedTempFile};

    #[test]
    fn parses_folder() {
        let dir = Builder::new()
            .prefix("my-folder")
            .rand_bytes(0)
            .tempdir()
            .unwrap();

        let item = NbItem::parse(0, dir.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, "my-folder");
        assert_eq!(item.kind, NbItemKind::Folder);
        assert!(!item.pinned);
        assert!(!item.encrypted);
    }

    #[test]
    fn parses_pinned_folder() {
        let dir = Builder::new()
            .prefix("my-folder")
            .rand_bytes(0)
            .tempdir()
            .unwrap();
        let pinned: HashSet<String> = ["my-folder".to_string()].into_iter().collect();

        let item = NbItem::parse(0, dir.path().to_path_buf(), &pinned).unwrap();

        assert!(item.pinned);
    }

    #[test]
    fn parses_undone_todo() {
        let mut file = NamedTempFile::with_suffix(".todo.md").unwrap();

        file.write(b"# [ ] Water the plants\n").unwrap();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, "Water the plants");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
        assert!(!item.encrypted);
    }

    #[test]
    fn parses_done_todo() {
        let mut file = NamedTempFile::with_suffix(".todo.md").unwrap();

        file.write(b"# [x] Water the plants\n").unwrap();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.kind, NbItemKind::Todo { done: true });
    }

    #[test]
    fn parses_bookmark() {
        let mut file = NamedTempFile::with_suffix(".bookmark.md").unwrap();

        file.write(b"# Rust Docs (doc.rust-lang.org)\n\n<https://doc.rust-lang.org>")
            .unwrap();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

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
        let mut file = NamedTempFile::with_suffix(".md").unwrap();

        file.write(b"# Just a regular note\n\nSome body text.\n")
            .unwrap();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, "Just a regular note");
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn note_without_heading_falls_back_to_filename() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();

        file.write(b"no heading here\n").unwrap();
        let file_name = file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, file_name);
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn parses_encrypted_todo() {
        let mut file = NamedTempFile::with_suffix(".todo.md.enc").unwrap();

        file.write(b"# [ ] Secret task\n").unwrap();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert!(item.encrypted);
        assert_eq!(item.title, "Secret task");
        assert_eq!(item.kind, NbItemKind::Todo { done: false });
    }

    #[test]
    fn classifies_source_file_as_note_using_filename_as_title() {
        let mut file = NamedTempFile::with_suffix(".rs").unwrap();

        file.write(b"fn main() {}\n").unwrap();
        let file_name = file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, file_name);
        assert_eq!(item.kind, NbItemKind::Note);
    }

    #[test]
    fn classifies_image_extension() {
        let file = NamedTempFile::with_suffix(".png").unwrap();

        // image kind never reads the file, so no write needed
        let file_name = file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let item = NbItem::parse(0, file.path().to_path_buf(), &HashSet::new()).unwrap();

        assert_eq!(item.title, file_name);
        assert_eq!(item.kind, NbItemKind::Image);
    }

    #[test]
    fn respects_pinned_set_for_files() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();

        file.write(b"# A note\n").unwrap();
        let file_name = file
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let pinned: HashSet<String> = [file_name].into_iter().collect();

        let item = NbItem::parse(0, file.path().to_path_buf(), &pinned).unwrap();

        assert!(item.pinned);
    }

    #[test]
    fn unpinned_file_not_in_pindex() {
        let mut file = NamedTempFile::with_suffix(".md").unwrap();

        file.write(b"# A note\n").unwrap();
        let pinned: HashSet<String> = ["some-other-file.md".to_string()].into_iter().collect();

        let item = NbItem::parse(0, file.path().to_path_buf(), &pinned).unwrap();

        assert!(!item.pinned);
    }
}
