use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::{
    app::AppEvent,
    nb::{self, NbRoot},
};

pub struct NotebookState {
    pub notebooks: Vec<String>,
    pub list_state: ListState,
    pub current_notebook: String,
    pub switchable: bool,
}

impl NotebookState {
    pub fn new(nb_root: &NbRoot) -> Self {
        match nb_root {
            NbRoot::Local(path) => {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| path.display().to_string());

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

    pub fn refresh(&mut self, nb_root: &NbRoot) {
        if let NbRoot::Global(root) = nb_root {
            self.notebooks = nb::get_notebooks(root);
        }
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
}
