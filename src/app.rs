use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use crossterm::event::{KeyCode, KeyEvent};
use notify::EventKind;

use crate::{
    nb::{self, FolderNav, NbRoot},
    ui::state::{
        items::ItemState,
        notebooks::NotebookState,
        search::{SearchMode, SearchState},
    },
};

#[derive(Clone, Copy, Debug)]
pub enum NbTag {
    RefreshItems,
    RemoveItem,
}

pub enum AppEvent {
    None,
    Quit,
    OpenEditor(usize),
    NotebookSelected(String),
    QueryChanged,
    SearchSubmitted,
    FolderOpened(String),
    FolderBack,
    ItemRemoved(usize),
    FsChanged,
}

pub enum CurrentSection {
    Notebooks,
    Items,
    Search,
}

pub struct App {
    pub current_section: CurrentSection,
    pub nb_root: NbRoot,
    pub nav: FolderNav,
    pub notebooks: NotebookState,
    pub items: ItemState,
    pub search: SearchState,
    fs_tx: Sender<EventKind>,
    fs_rx: Receiver<EventKind>,
    _watcher: notify::RecommendedWatcher,
}

impl App {
    pub fn default() -> Self {
        let explicit_path = std::env::args().nth(1).map(PathBuf::from);
        let (fs_tx, fs_rx) = mpsc::channel();

        let nb_root = nb::NbRoot::resolve(explicit_path).expect("could not resolve nb root");
        let nav = nb::FolderNav::new(nb_root.active_notebook_dir());

        let watcher = nb::spawn_fs_watcher(nb_root.watcher_root(), fs_tx.clone())
            .expect("failed to start fs watcher");

        let notebooks = NotebookState::new(&nb_root);

        let mut app = Self {
            current_section: CurrentSection::Notebooks,
            nb_root,
            nav,
            notebooks,
            items: ItemState::new(),
            search: SearchState::new(),
            fs_tx,
            fs_rx,
            _watcher: watcher,
        };

        app.refresh_items();
        app
    }

    fn refresh_items(&mut self) {
        match nb::scan_folder(&self.nav.current_dir()) {
            Ok(items) => {
                self.items.set_items(items);
                self.items.apply_filter(&self.search.query);
            }
            Err(_) => todo!(),
        }
    }

    pub fn poll_fs_events(&mut self) {
        let mut needs_refresh = false;
        while let Ok(event) = self.fs_rx.try_recv() {
            match event {
                EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_) => {
                    needs_refresh = true
                }
                _ => {}
            }
        }

        if needs_refresh {
            self.refresh_items();
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> AppEvent {
        if !self.search.wants_raw_input() {
            // global key binds
            match key.code {
                KeyCode::Char('b') => {
                    self.current_section = CurrentSection::Notebooks;
                    return AppEvent::None;
                }
                KeyCode::Char('n') => {
                    self.current_section = CurrentSection::Items;
                    return AppEvent::None;
                }
                KeyCode::Char('s') => {
                    self.current_section = CurrentSection::Search;
                    self.search.mode = SearchMode::Editing;
                    return AppEvent::None;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return AppEvent::Quit;
                }
                _ => {}
            }
        }

        let action = match self.current_section {
            CurrentSection::Notebooks => self.notebooks.handle_key(key),
            CurrentSection::Items => self.items.handle_key(key),
            CurrentSection::Search => self.search.handle_key(key),
        };

        match action {
            AppEvent::QueryChanged => {
                self.items.apply_filter(&self.search.query);
            }
            AppEvent::SearchSubmitted => {
                self.items.apply_filter(&self.search.query);
                self.current_section = CurrentSection::Items;
            }
            AppEvent::FolderOpened(name) => {
                self.nav.enter(&name);
                self.search.clear();
                self.refresh_items();
            }
            AppEvent::FolderBack => {
                self.nav.go_back();
                self.search.clear();
                self.refresh_items();
            }
            other => {
                return other;
            }
        }
        AppEvent::None
    }
}
