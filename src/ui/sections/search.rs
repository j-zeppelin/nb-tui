use crossterm::event::{KeyCode, KeyEvent};

pub enum SearchMode {
    Editing,
    Navigating,
}

pub struct SearchState {
    pub query: String,
    pub mode: SearchMode,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            mode: SearchMode::Editing,
        }
    }

    pub fn wants_raw_input(&self) -> bool {
        matches!(self.mode, SearchMode::Editing)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.query.push(c);
                true
            }
            KeyCode::Backspace => {
                self.query.pop();
                true
            }
            KeyCode::Enter => {
                self.mode = SearchMode::Navigating;
                false
            }
            KeyCode::Esc => {
                self.query.clear();
                false
            }
            _ => false,
        }
    }
}
