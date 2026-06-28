use crossterm::event::{self};
use ratatui::DefaultTerminal;

use crate::app::{Action, App};

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

        if let Some(key_event) = event::read()?.as_key_press_event() {
            match app.handle_key_event(key_event) {
                Action::OpenEditor(id) => {
                    nb::open_in_editor(&mut terminal, id)?;
                    app.items.refresh();
                    app.items.apply_filter(&app.search.query);
                }
                Action::Quit => break Ok(()),
                _ => {}
            }
        }
    }
}
