use crossterm::event::{self, Event};
use ratatui::DefaultTerminal;

use crate::app::App;

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

        match event::read()? {
            Event::Key(key_event) => app.handle_key_event(key_event),
            _ => {}
        }

        if app.should_quit {
            break Ok(());
        }
    }
}
