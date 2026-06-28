use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::state::{
    items::ItemState,
    notebooks::NotebookState,
    search::{SearchEvent, SearchMode, SearchState},
};

pub enum CurrentSection {
    Notebooks,
    Items,
    Search,
}

pub struct App {
    pub should_quit: bool,
    pub current_section: CurrentSection,
    pub notebooks: NotebookState,
    pub items: ItemState,
    pub search: SearchState,
}

impl App {
    pub fn default() -> Self {
        Self {
            current_section: CurrentSection::Notebooks,
            should_quit: false,
            notebooks: NotebookState::new(),
            items: ItemState::new(),
            search: SearchState::new(),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if !self.search.wants_raw_input() {
            // global key binds
            match key.code {
                KeyCode::Char('b') => {
                    self.current_section = CurrentSection::Notebooks;
                    return;
                }
                KeyCode::Char('n') => {
                    self.current_section = CurrentSection::Items;
                    return;
                }
                KeyCode::Char('s') => {
                    self.current_section = CurrentSection::Search;
                    self.search.mode = SearchMode::Editing;
                    return;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return;
                }
                _ => {}
            }
        }

        // key binds per section
        match self.current_section {
            CurrentSection::Notebooks => self.notebooks.handle_key(key),
            CurrentSection::Items => self.items.handle_key(key),
            CurrentSection::Search => match self.search.handle_key(key) {
                SearchEvent::QueryChanged => self.items.apply_filter(&self.search.query),
                SearchEvent::Submitted => {
                    self.items.apply_filter(&self.search.query);
                    self.current_section = CurrentSection::Items;
                }
                SearchEvent::None => {}
            },
        }
    }
}
