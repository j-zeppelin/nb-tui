use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Offset, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::{
    nb::item::NbItemId,
    ui::popups::{confirm::ConfirmPopup, create::CreatePopup, error::ErrorPopup},
};

pub mod confirm;
pub mod create;
pub mod error;

#[derive(Debug, Default)]
struct Popup<'a, W: Widget> {
    title_top: Line<'a>,
    title_bottom: Line<'a>,
    content: W,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl<'a, W: Widget> Popup<'a, W> {
    pub fn new(content: W) -> Self {
        Self {
            content,
            title_top: Line::default(),
            title_bottom: Line::default(),
            border_style: Style::default(),
            title_style: Style::default(),
            style: Style::default(),
        }
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn title_bottom(mut self, title: Line<'a>) -> Self {
        self.title_bottom = title;
        self
    }

    pub fn title_top(mut self, title: Line<'a>) -> Self {
        self.title_top = title;
        self
    }
}

impl<W: Widget> Widget for Popup<'_, W> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);
        let block = Block::new()
            .title_top(self.title_top)
            .title_bottom(self.title_bottom.alignment(Alignment::Right))
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

pub fn centered_rect_top(width: u16, height: u16, offset_percentage: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(offset_percentage),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let target_row = vertical[1];

    // Horizontal: center a fixed-width rect using Flex::Center
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .flex(Flex::Center)
        .constraints([Constraint::Length(width)])
        .split(target_row);

    horizontal[0]
}

pub fn centered_rect_percent(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    area.centered(
        Constraint::Percentage(width_percent),
        Constraint::Percentage(height_percent),
    )
}

/// Darkens every cell's fg/bg in `area` by blending it toward black.
/// `factor` is 0.0 (no change) to 1.0 (fully black).
pub fn dim_area(buf: &mut Buffer, area: Rect) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            if !buf.area.contains((x, y).into()) {
                continue;
            }
            let cell = &mut buf[(x, y)];
            let style = cell.style().add_modifier(Modifier::DIM);
            cell.set_style(style);
        }
    }
}

pub enum Overlay {
    None,
    Error(ErrorPopup),
    Confirm(ConfirmPopup),
    NewNote(CreatePopup),
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
