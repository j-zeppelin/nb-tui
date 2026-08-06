use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{
    app::{App, AppEvent},
    nb::{NbItem, NbItemKind},
};

pub struct ItemState {
    pub list_state: ListState,
    items: Vec<NbItem>,
    visible_indices: Vec<usize>,
}

impl ItemState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            visible_indices: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn set_items(&mut self, items: Vec<NbItem>) {
        self.items = items;
        self.list_state.select(Some(0));
    }

    pub fn apply_filter(&mut self, query: &str) {
        self.visible_indices = if query.is_empty() {
            (0..self.items.len()).collect()
        } else {
            self.items
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    item.title
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                        .then_some(idx)
                })
                .collect()
        };
    }

    pub fn visible(&self) -> impl Iterator<Item = &NbItem> {
        self.visible_indices.iter().map(|&i| &self.items[i])
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.items.len()),
            None => 0,
        };

        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
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
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
            }
            KeyCode::Char('x') => {
                if let Some(idx) = self.list_state.selected()
                    && idx != 0
                    && let Some(item) = self.items.get(idx.saturating_sub(1))
                {
                    return AppEvent::ItemRemoved(item.id);
                };
            }
            KeyCode::Backspace => {
                return AppEvent::FolderBack;
            }
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    // go back to parent folder
                    if selected == 0 {
                        return AppEvent::FolderBack;
                    }

                    let selected_item = self.visible().nth(selected.saturating_sub(1)).cloned();

                    if let Some(item) = selected_item {
                        match item.kind {
                            NbItemKind::Folder => {
                                return AppEvent::FolderOpened(item.title);
                            }
                            _ => return AppEvent::OpenEditor(item.id),
                        }
                    }
                }
            }
            _ => {}
        };

        AppEvent::None
    }
}
