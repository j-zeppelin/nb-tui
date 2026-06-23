use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders},
};

use crate::app::{App, CurrentSection};

pub fn render(f: &mut Frame, app: &App) {
    let focused_blue = Style::new().blue();
    let unfocused = Style::default();

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
        .title("[1] Notebooks")
        .border_style(notebook_border);

    let note_border = Block::new()
        .borders(Borders::ALL)
        .title("[2] Notes")
        .border_style(note_border);

    let search_border = Block::new()
        .borders(Borders::ALL)
        .title("[S] Search")
        .border_style(search_border);

    f.render_widget(notebook_block, left_chunk);
    f.render_widget(search_border, right_chunks[0]);
    f.render_widget(note_border, right_chunks[1]);
}
