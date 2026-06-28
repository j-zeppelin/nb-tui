use crossterm::event::{KeyCode, KeyEvent};

pub enum SearchEvent {
    None,
    QueryChanged,
    Submitted,
}

pub enum SearchMode {
    Editing,
    Navigating,
}

pub struct SearchState {
    pub query: String,
    pub mode: SearchMode,
    pub character_idx: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            mode: SearchMode::Navigating,
            character_idx: 0,
        }
    }

    pub fn wants_raw_input(&self) -> bool {
        matches!(self.mode, SearchMode::Editing)
    }

    fn move_cursor_left(&mut self) {
        let cursor_pos = self.character_idx.saturating_sub(1);
        self.character_idx = self.clamp_cursor(cursor_pos);
    }

    fn move_cursor_right(&mut self) {
        let cursor_pos = self.character_idx.saturating_add(1);
        self.character_idx = self.clamp_cursor(cursor_pos);
    }

    fn clamp_cursor(&self, pos: usize) -> usize {
        pos.clamp(0, self.query.chars().count())
    }

    fn enter_char(&mut self, new_char: char) {
        let idx = self.byte_index();
        self.query.insert(idx, new_char);
        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        if self.character_idx != 0 {
            let current_idx = self.character_idx;

            let chars_before = self.query.chars().take(current_idx - 1);
            let chars_after = self.query.chars().skip(current_idx);

            self.query = chars_before.chain(chars_after).collect();
            self.move_cursor_left();
        }
    }

    fn byte_index(&self) -> usize {
        self.query
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_idx)
            .unwrap_or(self.query.len())
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SearchEvent {
        match key.code {
            KeyCode::Char(c) => {
                self.enter_char(c);
                SearchEvent::QueryChanged
            }
            KeyCode::Backspace => {
                self.delete_char();
                SearchEvent::QueryChanged
            }
            KeyCode::Enter => {
                self.mode = SearchMode::Navigating;
                SearchEvent::Submitted
            }
            KeyCode::Esc => {
                self.mode = SearchMode::Navigating;
                SearchEvent::None
            }

            _ => SearchEvent::None,
        }
    }
}
