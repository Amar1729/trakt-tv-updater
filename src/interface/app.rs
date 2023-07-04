use std::error;

use crate::{models::TraktShow, sources::imdb_reader};
use crossbeam::channel::unbounded;
use ratatui::widgets::{ScrollbarState, TableState};
use tui_input::Input;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Different modes for the app.
#[derive(Debug, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

/// Application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// Is the app accepting input?
    pub input: Input,
    pub mode: InputMode,

    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub items: Vec<TraktShow>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        App {
            running: true,
            ..Default::default()
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // should i create sender/receiver in new()?
        if self.items.len() == 0 {
            let (s, r) = unbounded();

            std::thread::spawn(move || s.send(imdb_reader::get_show_vec()).unwrap());

            match r.recv() {
                Ok(items) => {
                    self.scroll_state = self.scroll_state.content_length(items.len() as u16);
                    self.items = items;
                }
                Err(_) => {}
            }
        }
    }

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
