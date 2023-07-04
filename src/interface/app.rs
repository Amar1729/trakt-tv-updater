use std::error;

use crate::models::TraktShow;
use crossbeam::channel::{Receiver, Sender};
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
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// for communication with our data manager.
    pub sender_query: Sender<String>,
    pub receiver_rows: Receiver<Vec<TraktShow>>,

    /// Is the app accepting input?
    pub input: Input,
    pub mode: InputMode,

    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub items: Vec<TraktShow>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(s: Sender<String>, r: Receiver<Vec<TraktShow>>) -> Self {
        App {
            running: true,
            // removed #[derive(Default)] for App because s/r don't have sane defaults
            // (is there some builder pattern/crate i can use to reduce this?)
            sender_query: s,
            receiver_rows: r,

            input: Input::default(),
            mode: InputMode::Normal,
            table_state: TableState::default(),
            scroll_state: ScrollbarState::default(),
            items: Vec::new(),
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // WIP implementation of query from our data rows
        // (right now, just pull everything on boot)
        if self.items.len() == 0 {
            self.sender_query.send(String::from("spurious")).unwrap();

            match self.receiver_rows.recv() {
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
