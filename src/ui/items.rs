use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
};

use crate::ui::{self, state::items::ItemState};

pub fn render(f: &mut Frame, area: Rect, state: &mut ItemState, focused: bool) {
    let border_style = ui::border_style(focused);

    let block = Block::new()
        .borders(Borders::ALL)
        .title("[n] Notes")
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    let mut items: Vec<ListItem> = state
        .visible()
        .map(|i| {
            let line = Line::from(vec![
                Span::styled(format!("[{}] ", i.id), Style::new().fg(Color::Green)),
                Span::raw(i.title.clone()),
            ]);

            ListItem::new(line)
        })
        .collect();

    items.insert(
        0,
        ListItem::new(
            Line::from(state.current_folder.to_string_lossy())
                .style(Style::default().bold().fg(Color::Blue).dim()),
        ),
    );

    let list = List::new(items)
        .highlight_style(if focused {
            Style::new().bold().bg(Color::DarkGray)
        } else {
            Style::default()
        })
        .highlight_symbol("> ")
        .block(block);

    f.render_stateful_widget(list, area, &mut state.list_state);
}
