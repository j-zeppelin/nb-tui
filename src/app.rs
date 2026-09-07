use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crossterm::event::{KeyCode, KeyEvent};
use notify::EventKind;

use crate::{
    config::Config,
    nb::{self, FolderNav, NbRoot},
    ui::{
        CurrentSection, Ui,
        items::ItemPanel,
        notebooks::NotebookPanel,
        popups::{Overlay, OverlayAction},
        search::{SearchMode, SearchPanel},
    },
};

#[allow(dead_code)]
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

#[allow(dead_code)]
pub struct App {
    pub nb_root: NbRoot,
    pub nav: FolderNav,
    pub notebooks: NotebookPanel,
    pub items: ItemPanel,
    pub search: SearchPanel,
    pub ui: Ui,
    pub config: Config,
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

        let watcher = nb::spawn_fs_watcher(nb_root.global_root(), fs_tx.clone())
            .expect("failed to start fs watcher");

        let notebooks = NotebookPanel::new(&nb_root);

        let mut app = Self {
            ui: Ui::default(),
            config: Config::load(),
            items: ItemPanel::new(),
            search: SearchPanel::new(),
            nb_root,
            nav,
            notebooks,
            fs_tx,
            fs_rx,
            _watcher: watcher,
        };

        app.refresh_items();
        app
    }

    pub fn poll_fs_events(&mut self) {
        let mut needs_refresh = false;
        while let Ok(event) = self.fs_rx.try_recv() {
            match event {
                EventKind::Create(_)
                | EventKind::Remove(_)
                | EventKind::Modify(_)
                | EventKind::Access(_) => needs_refresh = true,
                _ => {}
            }
        }

        if needs_refresh {
            self.refresh_items();
            self.refresh_notebooks();
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> AppEvent {
        if self.ui.overlay.is_active() {
            return self.handle_overlay_key(key);
        }

        if !self.search.wants_raw_input() {
            // global key binds
            match key.code {
                KeyCode::Char('b') => {
                    self.ui.current_section = CurrentSection::Notebooks;
                    return AppEvent::None;
                }
                KeyCode::Char('n') => {
                    self.ui.current_section = CurrentSection::Items;
                    return AppEvent::None;
                }
                KeyCode::Char('s') => {
                    self.ui.current_section = CurrentSection::Search;
                    self.search.mode = SearchMode::Editing;
                    return AppEvent::None;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return AppEvent::Quit;
                }
                _ => {}
            }
        }

        let action = match self.ui.current_section {
            CurrentSection::Notebooks => self.notebooks.handle_key(key),
            CurrentSection::Items => self.items.handle_key(key),
            CurrentSection::Search => self.search.handle_key(key),
        };

        match action {
            AppEvent::NotebookSelected(notebook) => {
                match nb::set_current_notebook(self.nb_root.global_root(), &notebook) {
                    Ok(_) => {
                        self.nav.reset(self.nb_root.global_root().join(&notebook));
                        self.notebooks.current_notebook = notebook;
                        self.refresh_items();
                    }
                    Err(err) => self.ui.display_err(err.to_string()),
                }
            }

            AppEvent::QueryChanged => {
                self.items.apply_filter(&self.search.query);
            }
            AppEvent::SearchSubmitted => {
                self.items.apply_filter(&self.search.query);
                self.ui.current_section = CurrentSection::Items;
            }
            AppEvent::FolderOpened(name) => {
                self.nav.enter(&name);
                self.search.clear();
                self.refresh_items();
            }
            AppEvent::FolderBack => {
                if !self.nav.is_at_root() {
                    self.nav.go_back();
                    self.search.clear();
                    self.refresh_items();
                }
            }
            other => {
                return other;
            }
        }
        AppEvent::None
    }

    pub fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(10), Constraint::Min(1)].as_ref())
            .split(f.area());

        let left_chunk = chunks[0];
        let right_chunk = chunks[1];

        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(right_chunk);

        self.notebooks.render(
            f,
            left_chunk,
            matches!(self.ui.current_section, CurrentSection::Notebooks),
        );

        self.search.render(
            f,
            right_chunks[0],
            matches!(self.ui.current_section, CurrentSection::Search),
        );

        self.items.render(
            f,
            right_chunks[1],
            &self.nav,
            &self.config,
            matches!(self.ui.current_section, CurrentSection::Items),
        );

        match &self.ui.overlay {
            Overlay::None => {}
            Overlay::Error(error_popup) => error_popup.render(f, f.area()),
            Overlay::Confirm => todo!(),
            Overlay::NewNote => todo!(),
        }
    }

    fn refresh_items(&mut self) {
        match nb::scan_folder(&self.nav.current_dir()) {
            Ok(items) => {
                self.items.set_items(items);
                self.items.apply_filter(&self.search.query);
            }
            Err(err) => self.ui.display_err(err.to_string()),
        }
    }

    fn refresh_notebooks(&mut self) {
        let root = self.nb_root.global_root();
        let notebooks = nb::get_notebooks(root);
        self.notebooks
            .set_notebooks(notebooks, nb::get_current_notebook(root));
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> AppEvent {
        let action = match &mut self.ui.overlay {
            Overlay::Error(error_popup) => error_popup.handle_key(key),
            Overlay::Confirm => todo!(),
            Overlay::NewNote => todo!(),
            Overlay::None => unreachable!(),
        };

        match action {
            OverlayAction::Close => self.ui.close_overlay(),
            OverlayAction::Confirm(confirm_action) => {
                self.ui.close_overlay();
                // TODO handle confirm
            }
            OverlayAction::None => {}
        }
        return AppEvent::None;
    }
}
