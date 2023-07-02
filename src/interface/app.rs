use std::error;

use crate::sources::imdb_reader::ImdbShow;
use ratatui::widgets::TableState;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub state: TableState,
    pub items: Vec<ImdbShow>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,

            state: TableState::default().with_selected(Some(0)),
            items: vec![],
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(items: Vec<ImdbShow>) -> Self {
        let mut app = Self::default();

        // TODO: i should instead query items from imdb_reader(?) during tick()
        app.items = items;

        app
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn next(&mut self, _step: Option<usize>) {
        // TODO: _step for larger jumps (but i don't want them to wrap around)
        // let bound = self.items.len();
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn prev(&mut self, _step: Option<usize>) {
        // TODO: _step for larger jumps (but i don't want them to wrap around)
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.items.len() - 1,
        };
        self.state.select(Some(i));
    }
}
