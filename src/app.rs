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
    nav::FolderNav,
    nb::{self, item::NbItemId, root::NbRoot},
    ui::{
        CurrentSection, Ui,
        items::ItemPanel,
        notebooks::NotebookPanel,
        popups::{ConfirmAction, Overlay, OverlayAction, confirm::ConfirmPopup, error::ErrorPopup},
        search::{SearchMode, SearchPanel},
    },
};

#[allow(dead_code)]
pub enum AppEvent {
    None,
    Quit,
    OpenEditor(NbItemId),
    RequestDelete(NbItemId),
    RequestCreate,
    NotebookSelected(String),
    QueryChanged,
    SearchSubmitted,
    FolderOpened(String),
    FolderBack,
    FsChanged,
}

#[allow(dead_code)]
pub struct App {
    pub nb_root: NbRoot,
    pub nav: FolderNav,
    pub notebook_panel: NotebookPanel,
    pub item_panel: ItemPanel,
    pub search_panel: SearchPanel,
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

        let nb_root = NbRoot::resolve(explicit_path).expect("could not resolve nb root");
        let nav = FolderNav::new(nb_root.active_notebook_dir());

        let watcher = nb::spawn_fs_watcher(nb_root.global_root(), fs_tx.clone())
            .expect("failed to start fs watcher");

        let notebook_panel = NotebookPanel::new(&nb_root);

        let mut app = Self {
            ui: Ui::default(),
            config: Config::load(),
            item_panel: ItemPanel::new(),
            search_panel: SearchPanel::new(),
            nb_root,
            nav,
            notebook_panel,
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

        if !self.search_panel.wants_raw_input() {
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
                    self.search_panel.mode = SearchMode::Editing;
                    return AppEvent::None;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return AppEvent::Quit;
                }
                _ => {}
            }
        }

        let action = match self.ui.current_section {
            CurrentSection::Notebooks => self.notebook_panel.handle_key(key),
            CurrentSection::Items => self.item_panel.handle_key(key),
            CurrentSection::Search => self.search_panel.handle_key(key),
        };

        match action {
            AppEvent::NotebookSelected(notebook) => {
                match nb::root::set_current_notebook(self.nb_root.global_root(), &notebook) {
                    Ok(..) => {
                        self.nav.reset(self.nb_root.global_root().join(&notebook));
                        self.notebook_panel.current_notebook = notebook;
                        self.refresh_items();
                    }
                    Err(err) => self.ui.display_err(err.to_string()),
                }
            }

            AppEvent::QueryChanged => {
                self.item_panel.apply_filter(&self.search_panel.query);
            }
            AppEvent::SearchSubmitted => {
                self.item_panel.apply_filter(&self.search_panel.query);
                self.ui.current_section = CurrentSection::Items;
            }
            AppEvent::FolderOpened(name) => {
                self.nav.enter(&name);
                self.search_panel.clear();
                self.refresh_items();
            }
            AppEvent::FolderBack => {
                if !self.nav.is_at_root() {
                    self.nav.go_back();
                    self.search_panel.clear();
                    self.refresh_items();
                }
            }
            AppEvent::RequestDelete(id) => {
                let item = self.item_panel.items.iter().find(|i| i.id == id);

                if let Some(item) = item {
                    self.ui.overlay = Overlay::Confirm(ConfirmPopup {
                        message: format!("Delete {} ({})?", item.title, item.filename),
                        selected: crate::ui::popups::confirm::Choice::Yes,
                        on_confirm: ConfirmAction::DeleteItem(item.id.clone()),
                    });
                } else {
                    self.ui
                        .display_err(format!("Could not find item with id {id}!"));
                }
            }
            AppEvent::RequestCreate => self.ui.overlay = Overlay::NewNote(todo!()),
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

        self.notebook_panel.render(
            f,
            left_chunk,
            matches!(self.ui.current_section, CurrentSection::Notebooks),
        );

        self.search_panel.render(
            f,
            right_chunks[0],
            matches!(self.ui.current_section, CurrentSection::Search),
        );

        self.item_panel.render(
            f,
            right_chunks[1],
            &self.nav,
            &self.config,
            matches!(self.ui.current_section, CurrentSection::Items),
        );

        match &self.ui.overlay {
            Overlay::None => {}
            Overlay::Error(error_popup) => error_popup.render(f, f.area()),
            Overlay::Confirm(confirm_popup) => confirm_popup.render(f, f.area()),
            Overlay::NewNote => todo!(),
        }
    }

    fn refresh_items(&mut self) {
        let items = nb::item::scan_folder(&self.nav.current_dir());
        self.item_panel.set_items(items);
        self.item_panel.apply_filter(&self.search_panel.query);
    }

    fn add_item(&mut self, name: String, encrypted: bool, pinned: bool) {
        todo!()
    }

    fn remove_item(&mut self, id: &NbItemId) {
        if let Err(err) = nb::remove_item(id) {
            self.ui.display_err(err.to_string());
        } else {
            self.refresh_items();
        }
    }

    fn refresh_notebooks(&mut self) {
        let root = self.nb_root.global_root();
        let notebooks = nb::root::get_notebooks(root);
        self.notebook_panel
            .set_notebooks(&notebooks, &nb::root::get_current_notebook(root));
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> AppEvent {
        let action = match &mut self.ui.overlay {
            Overlay::Error(_) => ErrorPopup::handle_key(key),
            Overlay::Confirm(confirm_popup) => confirm_popup.handle_key(key),
            Overlay::NewNote(create_popup) => create_popup.handle_key(key),
            Overlay::None => unreachable!(),
        };

        match action {
            OverlayAction::Close => self.ui.close_overlay(),
            OverlayAction::Confirm(confirm_action) => {
                self.ui.close_overlay();

                match confirm_action {
                    ConfirmAction::DeleteItem(id) => self.remove_item(&id),
                    ConfirmAction::CreateNote {
                        name,
                        encrypted,
                        pinned,
                    } => self.add_item(name, encrypted, pinned),
                }
            }

            OverlayAction::None => {}
        }

        AppEvent::None
    }
}
