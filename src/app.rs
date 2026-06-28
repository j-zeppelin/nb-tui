use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::state::{
    items::ItemState,
    notebooks::NotebookState,
    search::{SearchMode, SearchState},
};

pub enum Action {
    None,
    Quit,
    OpenEditor(usize),
    QueryChanged,
    SearchSubmitted,
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if !self.search.wants_raw_input() {
            // global key binds
            match key.code {
                KeyCode::Char('b') => {
                    self.current_section = CurrentSection::Notebooks;
                    return Action::None;
                }
                KeyCode::Char('n') => {
                    self.current_section = CurrentSection::Items;
                    return Action::None;
                }
                KeyCode::Char('s') => {
                    self.current_section = CurrentSection::Search;
                    self.search.mode = SearchMode::Editing;
                    return Action::None;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Action::Quit;
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
            Action::QueryChanged => {
                self.items.apply_filter(&self.search.query);
                Action::None
            }
            Action::SearchSubmitted => {
                self.items.apply_filter(&self.search.query);
                self.current_section = CurrentSection::Items;
                Action::None
            }
            other => other,
        }
    }
}
