use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::ui;
use crate::{
    app::AppEvent,
    nb::{self, NbRoot},
};

pub struct NotebookPanel {
    pub notebooks: Vec<String>,
    pub list_state: ListState,
    pub current_notebook: String,
    pub switchable: bool,
}

impl NotebookPanel {
    pub fn new(nb_root: &NbRoot) -> Self {
        match nb_root {
            NbRoot::Local(path) => {
                let name = path.file_name().map_or_else(
                    || path.display().to_string(),
                    |n| n.to_string_lossy().into_owned(),
                );

                Self {
                    notebooks: vec![name.clone()],
                    current_notebook: name,
                    list_state: ListState::default().with_selected(Some(0)),
                    switchable: false,
                }
            }
            NbRoot::Global(root) => {
                let notebooks = nb::get_notebooks(root);
                let current_notebook = nb::get_current_notebook(root);

                Self {
                    notebooks,
                    current_notebook,
                    list_state: ListState::default().with_selected(Some(0)),
                    switchable: true,
                }
            }
        }
    }

    pub fn set_notebooks(&mut self, notebooks: &[String], current: &str) {
        notebooks.clone_into(&mut self.notebooks);

        self.list_state
            .select(notebooks.iter().position(|n| *n == current));
    }

    fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => (i + 1).min(self.notebooks.len().saturating_sub(1)),
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

    fn selected_name(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.notebooks.get(i))
            .map(String::as_str)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AppEvent {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous();
            }
            KeyCode::Enter if self.switchable => match self.selected_name() {
                Some(name) if name != self.current_notebook => {
                    return AppEvent::NotebookSelected(name.to_string());
                }
                _ => {}
            },
            _ => {}
        }
        AppEvent::None
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let border_style = ui::border_style(focused);

        let notebooks: Vec<ListItem> = self
            .notebooks
            .iter()
            .map(|n| {
                let style = if *n == self.current_notebook {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                ListItem::new(n.clone()).style(style)
            })
            .collect();

        let block = Block::new()
            .borders(Borders::ALL)
            .title("[b] Notebooks")
            .border_style(border_style)
            .border_type(BorderType::Rounded);

        let list = List::new(notebooks)
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
