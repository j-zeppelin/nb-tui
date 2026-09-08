use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::popups::{ConfirmAction, OverlayAction, Popup, centered_rect_fixed};

#[derive(PartialEq, Eq)]
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
        match key.code {
            KeyCode::Char('h') | KeyCode::Left => self.selected = Choice::Yes,
            KeyCode::Char('l') | KeyCode::Right => self.selected = Choice::No,
            KeyCode::Tab => {
                if self.selected == Choice::Yes {
                    self.selected = Choice::No;
                } else {
                    self.selected = Choice::Yes;
                }
            }
            KeyCode::Enter => match self.selected {
                Choice::Yes => {
                    return OverlayAction::Confirm(self.on_confirm.clone());
                }
                Choice::No => {
                    return OverlayAction::Close;
                }
            },
            KeyCode::Esc | KeyCode::Char('q') => {
                return OverlayAction::Close;
            }

            KeyCode::Char('y') => return OverlayAction::Confirm(self.on_confirm.clone()),
            KeyCode::Char('n') => return OverlayAction::Close,
            _ => {}
        }

        OverlayAction::None
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let content = Paragraph::new(vec![
            Line::from(self.message.clone()).bold(),
            Line::from(""),
            Line::from(vec![
                Span::from("Yes (y)").add_modifier(if self.selected == Choice::Yes {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                }),
                Span::from("     "),
                Span::from("No (n)").add_modifier(if self.selected == Choice::No {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                }),
            ]),
        ])
        .alignment(Alignment::Center);

        let popup = Popup::new("Deletion", content).border_style(Style::new().yellow());

        f.render_widget(popup, centered_rect_fixed(50, 5, area));
    }
}
