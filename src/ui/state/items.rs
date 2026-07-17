use std::{path::PathBuf, vec};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{
    app::AppEvent,
    nb::{NbItem, NbItemKind},
};

pub struct ItemState {
    pub list_state: ListState,
    pub current_folder: PathBuf,
    items: Vec<NbItem>,
    visible_indices: Vec<usize>,
}

impl ItemState {
    pub fn new() -> Self {
        Self {
            current_folder: PathBuf::from("/"),
            items: Vec::new(),
            visible_indices: Vec::new(),
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn set_items_from_str(&mut self, stdout: String) {
        self.items = stdout.lines().map_while(|l| NbItem::parse(l)).collect();
        self.list_state.select(Some(0));
    }

    pub fn ls_args(&self) -> Vec<String> {
        let mut path = self.current_folder.to_path_buf();

        if let Ok(stripped) = path.strip_prefix("/") {
            path = stripped.to_path_buf();
        }
        let path = path.to_string_lossy() + "/";

        vec!["ls", &path, "--no-header", "--no-footer", "-af"]
            .iter()
            .map(ToString::to_string)
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
            KeyCode::Enter => {
                if let Some(selected) = self.list_state.selected() {
                    // go back to parent folder
                    if selected == 0 {
                        self.current_folder.pop();
                        return AppEvent::FolderOpened;
                    }

                    let selected_item = self.visible().nth(selected.saturating_sub(1)).cloned();

                    if let Some(item) = selected_item {
                        match item.kind {
                            NbItemKind::Folder => {
                                self.current_folder.push(&format!("{}", item.title));
                                return AppEvent::FolderOpened;
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
