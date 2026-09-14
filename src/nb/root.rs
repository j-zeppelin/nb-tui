use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::nb::is_ignored;

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

pub fn get_notebooks(nb_root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(nb_root) else {
        return vec!["home".to_string()];
    };

    entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type()
                .is_ok_and(|t| t.is_dir() && !is_ignored(&e.path()))
        })
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect()
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
