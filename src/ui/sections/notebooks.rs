use ratatui::widgets::ListState;

use crate::nb;

pub struct NotebookState {
    pub notebooks: Vec<String>,
    pub list_state: ListState,
}

impl NotebookState {
    pub fn new() -> Self {
        Self {
            notebooks: Self::fetch(),
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
            Some(i) => (i.saturating_sub(1)),
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

