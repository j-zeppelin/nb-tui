use crossterm::event::{KeyCode, KeyEvent};

pub enum CurrentSection {
    Notebooks,
    Notes,
    Search,
}

pub struct App {
    pub should_quit: bool,
    pub current_section: CurrentSection,
}

impl App {
    pub fn default() -> Self {
        Self {
            current_section: CurrentSection::Notebooks,
            should_quit: false,
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('b') => self.current_section = CurrentSection::Notebooks,
            KeyCode::Char('n') => self.current_section = CurrentSection::Notes,
            KeyCode::Char('s') => self.current_section = CurrentSection::Search,
            KeyCode::Esc => self.should_quit = true,
            _ => {}
        }
    }
}
