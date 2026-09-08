use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Paragraph, Wrap},
};

use crate::ui::popups::{OverlayAction, Popup, centered_rect_percent};

pub struct ErrorPopup {
    pub message: String,
}

impl ErrorPopup {
    pub fn handle_key(key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => OverlayAction::Close,
            _ => OverlayAction::None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let popup = Popup::new(
            "Error",
            Paragraph::new(self.message.as_str())
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center),
        )
        .border_style(Style::new().red());

        f.render_widget(popup, centered_rect_percent(50, 10, area));
    }
}
