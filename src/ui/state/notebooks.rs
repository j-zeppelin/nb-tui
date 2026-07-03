use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{app::AppEvent, nb};

pub struct NotebookState {
    pub notebooks: Vec<String>,
    pub list_state: ListState,
    pub current_notebook: String,
}

impl NotebookState {
    pub fn new() -> Self {
        let notebooks = Self::fetch();

        let current_notebook = notebooks
            .first()
            .cloned()
            .expect("one notebook must always exist");

        Self {
            notebooks,
            current_notebook,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    fn fetch() -> Vec<String> {
        nb::execute(["notebooks"])
            .unwrap()
            .lines()
            .map(ToString::to_string)
            .collect()
    }

    pub fn refresh(&mut self) {
        self.notebooks = Self::fetch()
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.notebooks.len().saturating_sub(1)),
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppEvent {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
                AppEvent::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }
}
