use std::time::Duration;

use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::app::{App, AppEvent};

mod app;
mod nb;
mod ui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    if let Err(err) = nb::check_nb_available() {
        eprintln!("{err}");
        std::process::exit(1);
    }

    let terminal = ratatui::init();
    let mut app = App::default();

    run(terminal, &mut app)?;
    ratatui::restore();

    Ok(())
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> color_eyre::Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match app.handle_key_event(key_event) {
                    AppEvent::OpenEditor(id) => {}
                    AppEvent::Quit => break Ok(()),
                    _ => {}
                }
            }
        }

        app.poll_fs_events();
    }
}
