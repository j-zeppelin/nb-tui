use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::ui::{self, sections::search::SearchState};

pub fn render(f: &mut Frame, area: Rect, state: &mut SearchState, focused: bool) {
    let border_style = ui::border_style(focused);

    let block = Block::new()
        .borders(Borders::ALL)
        .title("[s] Search")
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let text = Paragraph::new(state.query.clone()).block(block);

    f.render_widget(text, area);
}
