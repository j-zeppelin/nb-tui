use std::{
    collections::HashMap,
    io::Result,
    path::{Path, PathBuf},
    process::Command,
};

use crate::nb::item::NbItemKind;

#[derive(Debug, Clone)]
pub struct Config {
    pub editor: String,
    pub indicators: Indicators,
}

impl Config {
    pub fn load() -> Self {
        let vars = source_nbrc(&find_nbrc()).unwrap_or_default();

        let editor = vars
            .get("NB_EDITOR")
            .or_else(|| vars.get("EDITOR"))
            .cloned()
            .unwrap_or("nano".to_string());

        Self {
            editor,
            indicators: Indicators::from_env(&vars),
        }
    }
}

fn find_nbrc() -> PathBuf {
    std::env::var("NBRC_PATH").map_or(PathBuf::from("~/.nbrc"), PathBuf::from)
}

fn source_nbrc(path: &Path) -> Result<HashMap<String, String>> {
    let script = format!("source {} 2>/dev/null && env -0", path.display());

    let cmd = Command::new("bash").arg("-c").arg(script).output()?;

    let mut vars = HashMap::new();

    for entry in cmd.stdout.split(|b| *b == 0) {
        if entry.is_empty() {
            continue;
        }

        if let Ok(s) = str::from_utf8(entry)
            && let Some((v, k)) = s.split_once('=')
        {
            vars.insert(k.to_string(), v.to_string());
        }
    }

    Ok(vars)
}

const DEFAULT_INDICATORS: &[(&str, &str)] = &[
    ("NB_INDICATOR_AUDIO", "🔉"),
    ("NB_INDICATOR_BOOKMARK", "🔖"),
    ("NB_INDICATOR_DOCUMENT", "📄"),
    ("NB_INDICATOR_EBOOK", "📖"),
    ("NB_INDICATOR_ENCRYPTED", "🔒"),
    ("NB_INDICATOR_FOLDER", "📂"),
    ("NB_INDICATOR_IMAGE", "🌄"),
    ("NB_INDICATOR_PINNED", "📌"),
    ("NB_INDICATOR_TODO", "✔️"),
    ("NB_INDICATOR_TODO_DONE", "✅"),
    ("NB_INDICATOR_VIDEO", "📹"),
];

#[derive(Debug, Clone)]
pub struct Indicators {
    map: HashMap<&'static str, String>,
}

impl Indicators {
    pub fn from_env(vars: &HashMap<String, String>) -> Self {
        let map = DEFAULT_INDICATORS
            .iter()
            .map(|(key, default)| {
                let value = vars
                    .get(*key)
                    .cloned()
                    .unwrap_or_else(|| default.to_string());
                (*key, value)
            })
            .collect();

        Self { map }
    }

    pub fn for_kind(&self, kind: &NbItemKind) -> &str {
        match kind {
            NbItemKind::Note => "",
            NbItemKind::Bookmark { url: _ } => self.bookmark(),
            NbItemKind::Todo { done: true } => self.todo_done(),
            NbItemKind::Todo { done: false } => self.todo(),
            NbItemKind::Image => self.image(),
            NbItemKind::Audio => self.audio(),
            NbItemKind::Video => self.video(),
            NbItemKind::Document => self.document(),
            NbItemKind::Ebook => self.ebook(),
            NbItemKind::Folder => self.folder(),
        }
    }

    fn get(&self, key: &str) -> &str {
        self.map.get(key).map_or("", String::as_str)
    }

    pub fn folder(&self) -> &str {
        self.get("NB_INDICATOR_FOLDER")
    }
    pub fn document(&self) -> &str {
        self.get("NB_INDICATOR_DOCUMENT")
    }
    pub fn bookmark(&self) -> &str {
        self.get("NB_INDICATOR_BOOKMARK")
    }
    pub fn ebook(&self) -> &str {
        self.get("NB_INDICATOR_EBOOK")
    }
    pub fn image(&self) -> &str {
        self.get("NB_INDICATOR_IMAGE")
    }
    pub fn audio(&self) -> &str {
        self.get("NB_INDICATOR_AUDIO")
    }
    pub fn video(&self) -> &str {
        self.get("NB_INDICATOR_VIDEO")
    }
    pub fn todo(&self) -> &str {
        self.get("NB_INDICATOR_TODO")
    }
    pub fn todo_done(&self) -> &str {
        self.get("NB_INDICATOR_TODO_DONE")
    }
    pub fn pinned(&self) -> &str {
        self.get("NB_INDICATOR_PINNED")
    }
    pub fn encrypted(&self) -> &str {
        self.get("NB_INDICATOR_ENCRYPTED")
    }
}
