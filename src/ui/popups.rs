use ratatui::{
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, Widget},
};

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
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style)
            .style(self.style);

        let inner = block.inner(area);
        block.render(area, buf);
        self.content.render(inner, buf);
    }
}

pub enum OverlayAction {
    None,
    Close,
    Confirm(ConfirmAction),
}

#[derive(Clone)]
pub enum ConfirmAction {
    DeleteItem(usize),
    CreateNote {
        name: String,
        encrypted: bool,
        pinned: bool,
    },
}
