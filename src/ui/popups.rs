use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::{
    nb::item::NbItemId,
    ui::popups::{confirm::ConfirmPopup, error::ErrorPopup},
};

pub mod confirm;
pub mod error;

#[derive(Debug, Default)]
struct Popup<W: Widget> {
    title: Line<'static>,
    content: W,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl<W: Widget> Popup<W> {
    pub fn new(title: impl Into<Line<'static>>, content: W) -> Self {
        Self {
            title: title.into(),
            content,
            border_style: Style::default(),
            title_style: Style::default(),
            style: Style::default(),
        }
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }
}

impl<W: Widget> Widget for Popup<W> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .border_type(BorderType::Rounded)
            .style(self.style);

        let inner = block.inner(area);
        block.render(area, buf);
        self.content.render(inner, buf);
    }
}

pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    area.centered(Constraint::Max(width), Constraint::Max(height))
}

pub fn centered_rect_percent(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    area.centered(
        Constraint::Percentage(width_percent),
        Constraint::Percentage(height_percent),
    )
}

pub enum Overlay {
    None,
    Error(ErrorPopup),
    Confirm(ConfirmPopup),
    NewNote,
}

impl Overlay {
    pub fn is_active(&self) -> bool {
        !matches!(self, Overlay::None)
    }
}

pub enum OverlayAction {
    None,
    Close,
    Confirm(ConfirmAction),
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteItem(NbItemId),
    CreateNote {
        name: String,
        encrypted: bool,
        pinned: bool,
    },
}
