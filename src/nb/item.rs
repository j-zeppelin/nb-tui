use std::{
    cmp,
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub fn scan_folder(folder: &Path) -> Vec<NbItem> {
    let index = read_index(folder);
    let pinned = read_pindex(folder);

    let mut items: Vec<_> = index
        .into_iter()
        .filter_map(|(i, entry)| NbItem::parse(i + 1, folder.join(entry), &pinned))
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
            .enumerate()
            .filter_map(|(i, l)| (!l.is_empty()).then_some((i, l.to_string())))
            .collect(),
        // maybe create .index manually in the future
        Err(_) => HashMap::new(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbItemId {
    folder: Option<String>,
    id: usize,
}

impl NbItemId {
    pub fn new(folder: Option<String>, id: usize) -> Self {
        Self { folder, id }
    }

    pub fn from_path(path: &Path, id: usize) -> Self {
        let mut components = path.components();

        let folder = if components.by_ref().any(|c| c.as_os_str() == ".nb") {
            components.next();

            let rest: Vec<_> = components.filter_map(|c| c.as_os_str().to_str()).collect();

            let folder_parts = &rest[..rest.len().saturating_sub(1)];

            if folder_parts.is_empty() {
                None
            } else {
                Some(folder_parts.join("/"))
            }
        } else {
            None
        };

        Self { folder, id }
    }
}

impl std::fmt::Display for NbItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.folder {
            Some(folder) => write!(f, "{folder}/{}", self.id),
            None => write!(f, "{}", self.id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NbItem {
    pub id: NbItemId,
    pub title: String,
    pub filename: String,
    pub kind: NbItemKind,
    pub pinned: bool,
    pub encrypted: bool,
}

impl NbItem {
    pub fn parse(id: usize, entry: PathBuf, pinned: &HashSet<String>) -> Option<Self> {
        let id = NbItemId::from_path(&entry, id);
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
