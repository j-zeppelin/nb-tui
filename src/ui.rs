use ratatui::style::Style;

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
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().blue()
    } else {
        Style::new().blue().dim()
    }
}

pub enum Overlay {
    None,
    Error,
    Confirm,
    NewNote,
}

impl Overlay {
    pub fn is_active(&self) -> bool {
        !matches!(self, Overlay::None)
    }
}
