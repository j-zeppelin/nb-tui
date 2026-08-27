use std::{io, process::Command, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::DefaultTerminal;

use crate::app::{App, AppEvent};

mod app;
mod config;
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
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match app.handle_key_event(key_event) {
                    AppEvent::OpenEditor(id) => {
                        // TODO: handle error
                        let _ = open_in_editor(&mut terminal, id);
                    }
                    AppEvent::Quit => break Ok(()),
                    _ => {}
                }
            }
        }

        app.poll_fs_events();
    }
}

fn open_in_editor(term: &mut DefaultTerminal, id: usize) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(term.backend_mut(), LeaveAlternateScreen)?;

    Command::new("nb")
        .args(["edit", &id.to_string()])
        .status()?;

    enable_raw_mode()?;

    execute!(term.backend_mut(), EnterAlternateScreen)?;
    term.clear()?;
    Ok(())
}
