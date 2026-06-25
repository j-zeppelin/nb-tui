use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;

use crate::nb::{self, item::NbItem};

pub enum CurrentSection {
    Notebooks,
    Notes,
    Search,
}

pub struct App {
    pub should_quit: bool,
    pub current_section: CurrentSection,

    pub notebooks: Vec<String>,
    pub current_notebook: String,

    pub items: Vec<NbItem>,

    pub item_list_state: ListState,
}

impl App {
    pub fn default() -> Self {
        let output = nb::execute(["notebooks"]).unwrap();
        let notebooks: Vec<String> = output.lines().map(ToString::to_string).collect();
        assert!(!notebooks.is_empty());

        let current_notebook = notebooks[0].to_string();
        let items = nb::execute(["ls", "--no-header", "--no-footer", "-af"])
            .unwrap()
            .lines()
            .map_while(|l| NbItem::parse(l))
            .collect();

        Self {
            current_section: CurrentSection::Notebooks,
            should_quit: false,
            notebooks,
            current_notebook,
            items,
            item_list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('b') => self.current_section = CurrentSection::Notebooks,
            KeyCode::Char('n') => self.current_section = CurrentSection::Notes,
            KeyCode::Char('s') => self.current_section = CurrentSection::Search,
            KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn refresh_items(&mut self) {
        self.items = nb::execute(["ls", "--no-header", "--no-footer", "-af"])
            .unwrap()
            .lines()
            .map_while(|l| NbItem::parse(l))
            .collect();
    }
}
