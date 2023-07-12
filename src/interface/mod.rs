/// Application.
mod app;

/// Terminal events handler.
mod event;

/// Widget renderer.
mod ui;

/// Traits for converting db models to tui text/lines/cells
mod ui_traits;

/// Terminal user interface.
mod tui;

/// Event handler.
mod handler;

use crate::interface::{
    app::App,
    event::{Event, EventHandler},
    handler::{handle_key_events, handle_mouse_events},
    tui::Tui,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub async fn run() -> eyre::Result<()> {
    // Create an application.
    let app = App::new().await?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    let res = main_loop(app, &mut tui).await;

    if let Err(e) = &res {
        log::error!("App quit unexpectedly: {e:?}");
    }

    // Exit the user interface.
    tui.exit()?;
    res
}

async fn main_loop<B: ratatui::backend::Backend>(
    mut app: App,
    tui: &mut Tui<B>,
) -> eyre::Result<()> {
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(mouse_event) => handle_mouse_events(mouse_event, &mut app)?,
            Event::Resize(_, _) => {}
        }
    }
    Ok(())
}
