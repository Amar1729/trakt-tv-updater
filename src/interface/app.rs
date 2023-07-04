use std::error;

use crate::models::TraktShow;
use ratatui::widgets::{ScrollbarState, TableState};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub items: Vec<TraktShow>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,

            table_state: TableState::default().with_selected(Some(0)),
            scroll_state: ScrollbarState::default(),
            items: vec![],
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(items: Vec<TraktShow>) -> Self {
        let mut app = Self::default();

        app.scroll_state = app.scroll_state.content_length(items.len() as u16);
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

    pub fn next(&mut self, step: usize) {
        let i = match self.table_state.selected() {
            Some(i) => std::cmp::min(i + step, self.items.len() - 1),
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i as u16);
    }

    pub fn prev(&mut self, step: usize) {
        let i = match self.table_state.selected() {
            Some(i) => std::cmp::max(i as i32 - step as i32, 0) as usize,
            None => self.items.len() - 1,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i as u16);
    }
}
