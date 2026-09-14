use std::path::PathBuf;

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
        self.stack.is_empty()
    }

    pub fn breadcrumbs(&self) -> Vec<String> {
        self.stack
            .iter()
            .filter_map(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect()
    }

    pub fn nb_style_location(&self) -> String {
        self.breadcrumbs().join("/")
    }
}
