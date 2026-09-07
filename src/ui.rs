use ratatui::style::Style;

use crate::ui::popups::{Overlay, error::ErrorPopup};

pub mod items;
pub mod notebooks;
pub mod popups;
pub mod search;

pub enum CurrentSection {
    Notebooks,
    Items,
    Search,
}

pub struct Ui {
    pub current_section: CurrentSection,
    pub overlay: Overlay,
}

impl Ui {
    pub fn default() -> Self {
        Ui {
            current_section: CurrentSection::Notebooks,
            overlay: Overlay::None,
        }
    }

    pub fn display_err(&mut self, message: String) {
        self.overlay = Overlay::Error(ErrorPopup { message })
    }

    pub fn close_overlay(&mut self) {
        self.overlay = Overlay::None;
    }
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().blue()
    } else {
        Style::new().blue().dim()
    }
}
