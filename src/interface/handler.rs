use crate::interface::app::{App, AppMode, AppResult};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseEvent, MouseEventKind};

use tui_input::backend::crossterm::EventHandler;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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
            KeyCode::Char(' ') => app.toggle_watch_status(),
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
        _ => unimplemented!(),
    }

    Ok(())
}

// handle mouse events as well
pub fn handle_mouse_events(mouse_event: MouseEvent, app: &mut App) -> AppResult<()> {
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
