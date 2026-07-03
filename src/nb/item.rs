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
