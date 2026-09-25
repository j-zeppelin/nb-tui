use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{Paragraph, Wrap},
};

use crate::ui::popups::{OverlayAction, Popup, centered_rect_fixed};

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
            Paragraph::new(self.message.as_str())
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center),
        )
        .title_top("Error".into())
        .border_style(Style::new().red());

        f.render_widget(popup, centered_rect_fixed(50, 5, area));
    }
}
