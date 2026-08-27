use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
};

pub mod items;
pub mod notebooks;
pub mod search;
pub mod state;

pub enum CurrentSection {
    Notebooks,
    Items,
    Search,
}

pub struct Ui {
    pub current_section: CurrentSection,
}

impl Ui {
    pub fn default() -> Self {
        Ui {
            current_section: CurrentSection::Notebooks,
        }
    }
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().blue()
    } else {
        Style::new().blue().dim()
    }
}
