use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};
use tui_input::backend::crossterm::EventHandler;

use crate::{
    config::Indicators,
    nb::item::NbItemKind,
    ui::{
        border_style,
        popups::{ConfirmAction, OverlayAction, Popup, centered_rect_fixed, centered_rect_top},
    },
};

pub struct CreatePopup {
    pub input: tui_input::Input,
    pub on_create: ConfirmAction,
    encrypted: bool,
    pinned: bool,
}

impl CreatePopup {
    pub fn default() -> Self {
        Self {
            input: tui_input::Input::default().with_value("".to_string()),
            encrypted: false,
            pinned: false,
            on_create: ConfirmAction::CreateNote {
                name: "".to_string(),
                encrypted: false,
                pinned: false,
            },
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> OverlayAction {
        match key.code {
            KeyCode::Enter => return OverlayAction::Confirm(self.on_create.clone()),

            KeyCode::Esc => {
                return OverlayAction::Close;
            }
            _ => {
                self.input.handle_event(&Event::Key(key));
            }
        }

        OverlayAction::None
    }

    pub fn render(&self, f: &mut Frame, area: Rect, indicators: &Indicators) {
        let mut title_bottom = Line::default();
        if let Some((_, extension)) = self.input.value().rsplit_once(".") {
            let kind = NbItemKind::from_ext(extension);
            title_bottom.push_span(indicators.for_kind(&kind));
        }

        if self.encrypted {
            title_bottom.push_span(indicators.encrypted());
        }
        if self.pinned {
            title_bottom.push_span(indicators.pinned());
        }

        let input = Paragraph::new(self.input.value());
        let popup = Popup::new(input)
            .title_top("New Note".into())
            .title_bottom(title_bottom)
            .border_style(border_style(true));

        let popup_area = centered_rect_top(50, 3, 2, area);

        f.set_cursor_position((
            popup_area.x + self.input.visual_cursor() as u16 + 1,
            popup_area.y + 1,
        ));
        f.render_widget(popup, popup_area);
    }
}
