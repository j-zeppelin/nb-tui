use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, List},
};

use crate::app::{App, CurrentSection};

pub mod items;
pub mod notebooks;
pub mod search;
pub mod sections;

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

    notebooks::render(
        f,
        left_chunk,
        &mut app.notebooks,
        matches!(app.current_section, CurrentSection::Notebooks),
    );

    search::render(
        f,
        right_chunks[0],
        &mut app.search,
        matches!(app.current_section, CurrentSection::Search),
    );
    items::render(
        f,
        right_chunks[1],
        &mut app.items,
        matches!(app.current_section, CurrentSection::Items),
    );
}

pub fn border_style(focused: bool) -> Style {
    if focused {
        Style::new().blue()
    } else {
        Style::new().blue().dim()
    }
}
