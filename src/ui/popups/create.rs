use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::popups::{ConfirmAction, OverlayAction, Popup, centered_rect_fixed};

pub struct CreatePopup {
    pub input: tui_input::Input,
    pub on_create: ConfirmAction,
}

impl CreatePopup {
    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Enter => return OverlayAction::Confirm(self.on_create.clone()),

            KeyCode::Esc | KeyCode::Char('q') => {
                return OverlayAction::Close;
            }
            _ => {}
        }

        OverlayAction::None
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {

        // f.render_widget(popup, centered_rect_fixed(50, 5, area));
    }
}
