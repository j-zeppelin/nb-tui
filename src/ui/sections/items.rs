use ratatui::widgets::ListState;

use crate::nb::{self, item::NbItem};

pub struct ItemState {
    pub items: Vec<NbItem>,
    pub list_state: ListState,
}

impl ItemState {
    pub fn new() -> Self {
        Self {
            items: Self::fetch(),
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    fn fetch() -> Vec<NbItem> {
        nb::execute(["ls", "--no-header", "--no-footer", "-af"])
            .unwrap()
            .lines()
            .map_while(|l| NbItem::parse(l))
            .collect()
    }

    pub fn refresh(&mut self) {
        self.items = Self::fetch();
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.items.len().saturating_sub(1)),
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
