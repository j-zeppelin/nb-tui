use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::state::{
    items::ItemState,
    notebooks::NotebookState,
    search::{SearchMode, SearchState},
};

pub enum AppEvent {
    None,
    Quit,
    OpenEditor(usize),
    QueryChanged,
    SearchSubmitted,
    FolderOpened,
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
}

impl App {
    pub fn default() -> Self {
        Self {
            current_section: CurrentSection::Notebooks,
            notebooks: NotebookState::new(),
            items: ItemState::new(),
            search: SearchState::new(),
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
                self.items.refresh();
                self.items.apply_filter("");
                self.search.clear();

                AppEvent::None
            }
            other => other,
        }
    }
}
