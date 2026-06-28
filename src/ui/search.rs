use ratatui::{
    Frame,
    layout::{Position, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Paragraph},
};

use crate::ui::{
    self,
    state::search::{SearchMode, SearchState},
};

pub fn render(f: &mut Frame, area: Rect, state: &mut SearchState, focused: bool) {
    let border_style = ui::border_style(focused);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .style(match state.mode {
            SearchMode::Editing => Style::default().fg(Color::Yellow),
            SearchMode::Navigating => border_style,
        })
        .title("[s] Search");

    let input = Paragraph::new(state.query.as_str())
        .style(Style::default().fg(Color::Yellow).not_dim())
        .block(block);

    f.render_widget(input, area);

    match state.mode {
        SearchMode::Navigating => {}
        SearchMode::Editing => f.set_cursor_position(Position::new(
            area.x + state.character_idx as u16 + 1,
            area.y + 1,
        )),
    }
}
