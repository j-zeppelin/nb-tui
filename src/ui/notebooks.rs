use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List},
};

use crate::ui::{self, state::notebooks::NotebookState};

pub fn render(f: &mut Frame, area: Rect, state: &mut NotebookState, focused: bool) {
    let border_style = ui::border_style(focused);

    let block = Block::new()
        .borders(Borders::ALL)
        .title("[b] Notebooks")
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let list = List::new(state.notebooks.clone())
        .highlight_style(Style::new().bold().bg(Color::DarkGray))
        .highlight_symbol("> ")
        .block(block);

    f.render_stateful_widget(list, area, &mut state.list_state);
}
