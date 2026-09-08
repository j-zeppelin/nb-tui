use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::ui::popups::{ConfirmAction, OverlayAction};

pub enum Choice {
    Yes,
    No,
}

pub struct ConfirmPopup {
    pub message: String,
    pub selected: Choice,
    pub on_confirm: ConfirmAction,
}

impl ConfirmPopup {
    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        todo!()
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {}
}
