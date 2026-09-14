use std::{fs::File, io, process::Command, time::Duration};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::DefaultTerminal;
use tracing_appender::non_blocking::WorkerGuard;

use crate::{
    app::{App, AppEvent},
    nb::item::NbItemId,
};

mod app;
mod config;
mod nav;
mod nb;
mod ui;

fn init_logging() -> WorkerGuard {
    let file = File::create("debug.log").expect("failed to create log file");
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .init();

    guard
}

fn main() -> color_eyre::Result<()> {
    let _guard = init_logging();
    color_eyre::install()?;

    tracing::info!("app startring");

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

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key_event) = event::read()?
        {
            match app.handle_key_event(key_event) {
                AppEvent::OpenEditor(id) => {
                    // TODO: handle error
                    let _ = open_in_editor(&mut terminal, id);
                }
                AppEvent::Quit => break Ok(()),
                _ => {}
            }
        }

        app.poll_fs_events();
    }
}

fn open_in_editor(term: &mut DefaultTerminal, id: NbItemId) -> io::Result<()> {
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
