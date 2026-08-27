use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{app::AppEvent, ui};

pub enum SearchMode {
    Editing,
    Navigating,
}

pub struct SearchPanel {
    pub query: String,
    pub mode: SearchMode,
    pub character_idx: usize,
}

impl SearchPanel {
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

    pub fn clear(&mut self) {
        self.query.clear();
        self.character_idx = 0;
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

    pub fn handle_key(&mut self, key: KeyEvent) -> AppEvent {
        match key.code {
            KeyCode::Char(c) => {
                self.enter_char(c);
                AppEvent::QueryChanged
            }
            KeyCode::Backspace => {
                self.delete_char();
                AppEvent::QueryChanged
            }
            KeyCode::Enter => {
                self.mode = SearchMode::Navigating;
                AppEvent::SearchSubmitted
            }
            KeyCode::Esc => {
                self.mode = SearchMode::Navigating;
                AppEvent::None
            }
            KeyCode::Left => {
                self.move_cursor_left();
                AppEvent::None
            }
            KeyCode::Right => {
                self.move_cursor_right();
                AppEvent::None
            }

            _ => AppEvent::None,
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let border_style = ui::border_style(focused);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(match self.mode {
                SearchMode::Editing => Style::default().fg(Color::Yellow),
                SearchMode::Navigating => border_style,
            })
            .title("[s] Search");

        let input = Paragraph::new(self.query.as_str()).block(block);

        f.render_widget(input, area);

        match self.mode {
            SearchMode::Navigating => {}
            SearchMode::Editing => f.set_cursor_position(Position::new(
                area.x + self.character_idx as u16 + 1,
                area.y + 1,
            )),
        }
    }
}
