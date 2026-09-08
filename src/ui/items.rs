use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use crate::{
    app::AppEvent,
    config::Config,
    nb::{FolderNav, NbItem, NbItemKind},
    ui,
};

pub struct ItemPanel {
    pub list_state: ListState,
    pub items: Vec<NbItem>,
    visible_indices: Vec<usize>,
}

impl ItemPanel {
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
            KeyCode::Char('x') | KeyCode::Char('d') => {
                if let Some(idx) = self.list_state.selected()
                    && idx != 0
                    && let Some(item) = self.items.get(idx.saturating_sub(1))
                {
                    return AppEvent::RequestDelete(item.id);
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

    pub fn render(
        &mut self,
        f: &mut Frame,
        area: Rect,
        nav: &FolderNav,
        config: &Config,
        focused: bool,
    ) {
        let border_style = ui::border_style(focused);

        let block = Block::new()
            .borders(Borders::ALL)
            .title("[n] Notes")
            .border_style(border_style)
            .border_type(BorderType::Rounded);

        let mut items: Vec<ListItem> = self
            .visible()
            .map(|i| {
                let mut label = String::new();

                if i.pinned {
                    label.push_str(config.indicators.pinned());
                    label.push(' ');
                }

                if i.encrypted {
                    label.push_str(config.indicators.encrypted());
                    label.push(' ');
                }

                label.push_str(config.indicators.for_kind(&i.kind));
                if !matches!(i.kind, NbItemKind::Note) {
                    label.push(' ');
                }

                label.push_str(&i.title);

                let line = Line::from(vec![
                    Span::styled(format!("[{}] ", i.id), Style::default().green()),
                    Span::styled(
                        label,
                        if i.kind == NbItemKind::Folder {
                            Style::new().bold()
                        } else {
                            Style::new()
                        },
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        items.insert(
            0,
            ListItem::new(
                Line::from(format!("./{}", nav.breadcrumbs().join("/")))
                    .style(Style::default().bold().fg(Color::Blue).dim()),
            ),
        );

        let list = List::new(items)
            .highlight_style(if focused {
                Style::new().bold().bg(Color::DarkGray)
            } else {
                Style::default()
            })
            .highlight_symbol("> ")
            .block(block);

        f.render_stateful_widget(list, area, &mut self.list_state);
    }
}
