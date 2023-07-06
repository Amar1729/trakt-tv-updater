use crate::interface::app::{App, AppResult, AppMode};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseEvent, MouseEventKind};

use tui_input::backend::crossterm::EventHandler;

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.mode {
        AppMode::MainView => match key_event.code {
            KeyCode::Tab => {
                app.mode = AppMode::Querying;
            }
            _ => {}
        },
        AppMode::Querying => {
            match key_event.code {
                KeyCode::Enter => {
                    // TODO: do something else here? query rows for entered text?
                    app.mode = AppMode::MainView;
                }
                KeyCode::Tab => {
                    app.mode = AppMode::MainView;
                }
                KeyCode::Esc => {
                    app.mode = AppMode::MainView;
                }
                _ => {
                    app.input.handle_event(&CrosstermEvent::Key(key_event));
                }
            };

            // exit early (until i rework handler logic?)
            return Ok(());
        },
        _ => unimplemented!(),
    }

    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Up => {
            app.prev(1);
        }
        KeyCode::Down => {
            app.next(1);
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
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}

// handle mouse events as well
pub fn handle_mouse_events(mouse_event: MouseEvent, app: &mut App) -> AppResult<()> {
    match mouse_event.kind {
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
    }

    Ok(())
}
