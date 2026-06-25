use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List},
};

use crate::app::{App, CurrentSection};

pub fn render(f: &mut Frame, app: &mut App) {
    let focused_blue = Style::new().blue();
    let unfocused = Style::new().dark_gray();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Min(1)].as_ref())
        .split(f.area());

    let left_chunk = chunks[0];
    let right_chunk = chunks[1];

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(right_chunk);

    let (notebook_border, note_border, search_border) = match app.current_section {
        CurrentSection::Notebooks => (focused_blue, unfocused, unfocused),
        CurrentSection::Notes => (unfocused, focused_blue, unfocused),
        CurrentSection::Search => (unfocused, unfocused, focused_blue),
    };

    let notebook_block = Block::new()
        .borders(Borders::ALL)
        .title("[b] Notebooks")
        .border_style(notebook_border)
        .border_type(BorderType::Rounded);

    let notes_block = Block::new()
        .borders(Borders::ALL)
        .title("[n] Notes")
        .border_style(note_border)
        .border_type(BorderType::Rounded);

    let items: Vec<_> = app
        .items
        .iter()
        .map(|i| format!("[{}] {}", i.id, i.title))
        .collect();

    let notes_list = List::new(items)
        .highlight_style(Modifier::BOLD)
        .highlight_symbol("> ")
        .block(notes_block);

    let search_block = Block::new()
        .borders(Borders::ALL)
        .title("[s] Search")
        .border_style(search_border)
        .border_type(BorderType::Rounded);

    f.render_widget(notebook_block, left_chunk);
    f.render_widget(search_block, right_chunks[0]);
    f.render_stateful_widget(notes_list, right_chunks[1], &mut app.item_list_state);
}
