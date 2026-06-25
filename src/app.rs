use crossterm::event::{KeyCode, KeyEvent};

use crate::ui::sections::{items::ItemState, notebooks::NotebookState, search::SearchState};

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
        match key.code {
            KeyCode::Char('b') => self.current_section = CurrentSection::Notebooks,
            KeyCode::Char('n') => self.current_section = CurrentSection::Items,
            KeyCode::Char('s') => self.current_section = CurrentSection::Search,
            KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }
}
