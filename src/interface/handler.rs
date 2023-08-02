use crate::interface::app::{App, AppMode};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseEvent, MouseEventKind};

use tui_input::backend::crossterm::EventHandler;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> eyre::Result<()> {
    // Exit application from any mode on `Ctrl-C`
    match key_event.code {
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        _ => {}
    }

    match app.mode {
        AppMode::MainView => match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Esc | KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.prev(1);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.next(1);
            }
            KeyCode::PageUp => {
                app.prev(20);
            }
            KeyCode::PageDown => {
                app.next(20);
            }
            KeyCode::Char('u') | KeyCode::Char('U') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.prev(20);
                }
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.next(20);
                }
            }
            KeyCode::Char('g') => {
                app.table_state.select(Some(0));
            }
            KeyCode::Char('G') => {
                app.table_state.select(Some(app.shows.len() - 1));
            }
            // switch to query mode (search for shows in input bar)
            KeyCode::Tab => {
                app.mode = AppMode::Querying;
            }
            // cycle through watch status for a show
            KeyCode::Char(' ') => app.toggle_watch_status().await?,

            // open up tv show details view
            KeyCode::Char('l') | KeyCode::Right => {
                // app will only change its UI if a show is selected.
                app.enter_show_details().await?;
            }

            _ => {}
        },
        AppMode::Querying => match key_event.code {
            KeyCode::Enter => {
                // TODO: do something else here? query rows for entered text?
                app.mode = AppMode::MainView;
            }
            KeyCode::Tab | KeyCode::Esc => {
                app.mode = AppMode::MainView;
            }
            _ => {
                app.input.handle_event(&CrosstermEvent::Key(key_event));
            }
        },
        AppMode::SeasonView => match key_event.code {
            KeyCode::Left | KeyCode::Char('h') => app.mode = AppMode::MainView,
            KeyCode::Char('k') | KeyCode::Up => app.season_prev(1),
            KeyCode::Char('j') | KeyCode::Down => app.season_next(1),
            KeyCode::Char(' ') => app.toggle_season_watch_status().await?,
            _ => {}
        },
        _ => unimplemented!(),
    }

    Ok(())
}

// handle mouse events as well
pub fn handle_mouse_events(mouse_event: MouseEvent, app: &mut App) -> eyre::Result<()> {
    #[allow(clippy::single_match)]
    match app.mode {
        AppMode::MainView => match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                app.next(1);
                app.mode = AppMode::MainView;
            }
            MouseEventKind::ScrollUp => {
                app.prev(1);
                app.mode = AppMode::MainView;
            }
            // TODO: select a show if clicked
            MouseEventKind::Down(_) => {
                let _col = mouse_event.column;
                let row = mouse_event.row;

                if row == 0 {
                    app.mode = AppMode::Querying;
                } else if row > 1 {
                    // ... how do you get offset from table_state?
                }
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}
