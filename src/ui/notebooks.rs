use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::ui::{self, state::notebooks::NotebookState};

pub fn render(f: &mut Frame, area: Rect, state: &mut NotebookState, focused: bool) {
    let border_style = ui::border_style(focused);

    let notebooks: Vec<ListItem> = state
        .notebooks
        .iter()
        .map(|n| {
            let style = if *n == state.current_notebook {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            ListItem::new(n.file_name().unwrap().to_str().unwrap()).style(style)
        })
        .collect();

    let block = Block::new()
        .borders(Borders::ALL)
        .title("[b] Notebooks")
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let list = List::new(notebooks)
        .highlight_style(if focused {
            Style::new().bold().bg(Color::DarkGray)
        } else {
            Style::default()
        })
        .highlight_symbol("> ")
        .block(block);

    f.render_stateful_widget(list, area, &mut state.list_state);
}
