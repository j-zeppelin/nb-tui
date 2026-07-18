use std::sync::mpsc::{self, Receiver, Sender};

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    nb::{self, NbError},
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
    QueryChanged,
    SearchSubmitted,
    FolderOpened,
    ItemRemoved(usize),
}

pub enum CurrentSection {
    Notebooks,
    Items,
    Search,
}

pub struct App {
    pub current_section: CurrentSection,
    pub notebooks: NotebookState,
    pub items: ItemState,
    pub search: SearchState,
    nb_tx: Sender<(NbTag, Result<String, NbError>)>,
    nb_rx: Receiver<(NbTag, Result<String, NbError>)>,
}

impl App {
    pub fn default() -> Self {
        let (nb_tx, nb_rx) = mpsc::channel();
        let mut app = Self {
            current_section: CurrentSection::Notebooks,
            notebooks: NotebookState::new(),
            items: ItemState::new(),
            search: SearchState::new(),
            nb_tx,
            nb_rx,
        };

        app.refresh_items();
        app
    }

    fn refresh_items(&mut self) {
        let args = self.items.ls_args();
    }

    pub fn poll_nb_events(&mut self) {
        while let Ok((tag, result)) = self.nb_rx.try_recv() {
            match tag {
                NbTag::RefreshItems => match result {
                    Ok(stdout) => {
                        self.items.set_items_from_str(stdout);
                        self.items.apply_filter(&self.search.query);
                    }
                    Err(_) => todo!(),
                },
                NbTag::RemoveItem => {
                    self.refresh_items();
                }
            }
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
                AppEvent::None
            }
            AppEvent::SearchSubmitted => {
                self.items.apply_filter(&self.search.query);
                self.current_section = CurrentSection::Items;
                AppEvent::None
            }
            AppEvent::FolderOpened => {
                self.search.clear();
                self.refresh_items();

                AppEvent::None
            }
            AppEvent::ItemRemoved(id) => AppEvent::None,
            other => other,
        }
    }
}
