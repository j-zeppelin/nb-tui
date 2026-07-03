use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{
    app::{App, AppEvent},
    nb::{
        self,
        item::{NbItem, NbItemKind},
    },
};

pub struct ItemState {
    pub list_state: ListState,
    current_folder: String,
    items: Vec<NbItem>,
    visible_indices: Vec<usize>,
}

impl ItemState {
    pub fn new() -> Self {
        let items = Self::fetch(None);
        let visible_indices = (0..items.len()).collect();

        Self {
            current_folder: "".into(),
            items,
            visible_indices,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    fn fetch(path: Option<&str>) -> Vec<NbItem> {
        nb::execute([
            "ls",
            path.unwrap_or(""),
            "--no-header",
            "--no-footer",
            "-af",
        ])
        .unwrap()
        .lines()
        .map_while(|l| NbItem::parse(l))
        .collect()
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
        }
    }

    pub fn visible(&self) -> impl Iterator<Item = &NbItem> {
        self.visible_indices.iter().map(|&i| &self.items[i])
    }

    pub fn refresh(&mut self) {
        self.items = Self::fetch(Some(&self.current_folder));
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
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    let selected_item = self.visible().nth(selected).cloned();

                    if let Some(item) = selected_item {
                        match item.kind {
                            NbItemKind::Folder => {
                                self.current_folder.push_str(&format!("{}/", item.title));
                                return AppEvent::FolderOpened;
                            }
                            _ => return AppEvent::OpenEditor(item.id),
                        }
                    }
                }
                AppEvent::None
            }
            _ => AppEvent::None,
        }
    }
}
