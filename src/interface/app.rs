use std::error;

use crate::{
    models::{TraktShow, UserStatus},
    trakt::{t_api, t_db},
};
use crossbeam::channel::{unbounded, Receiver, SendError, Sender};
use log::*;
use ratatui::widgets::{ScrollbarState, TableState};
use reqwest::Client;
use tui_input::Input;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Different modes for the app.
#[derive(PartialEq, Eq, Debug, Default)]
pub enum AppMode {
    /// Various tasks to init the app (e.g. data pull + insert)
    #[default]
    Initializing,
    /// List of all the shows we find (from IMDB dataset / loaded from DB)
    MainView,
    /// somewhat of a todo state, i haven't impl'd searching yet
    Querying,
    /// Show keybindings
    HelpWindow,
    /// Detailed view of specific season
    SeasonView,
    // Detailed view of a specific episode
    // not sure about this one yet
    // EpisodeView,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// for communication with our data manager.
    pub sender_query: Sender<String>,
    pub receiver_rows: Receiver<Vec<TraktShow>>,

    /// for querying trakt
    pub client: Client,

    /// ui+handling changes based on the app's current view
    pub mode: AppMode,

    /// used in main view
    pub input: Input,
    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub shows: Vec<TraktShow>,

    // used in season view
    // TODO(?) maybe this should be a nested stuct, only relevant for season view?
    // similar with the stuff required by main view and eventual episode view
    pub show_details: Option<t_api::ApiShowDetails>,
    pub show_seasons: Vec<t_api::ApiSeasonDetails>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        // i think these both actually could be len0
        let (sq, receiver_query) = unbounded();
        let (sender_rows, rr) = unbounded();

        // when a new app is created, begin a bg data manager task
        // this task will receive a string query, and send back a TraktShow vec
        crate::sources::data_manager(sender_rows, receiver_query);

        App {
            running: true,
            // removed #[derive(Default)] for App because s/r don't have sane defaults
            // (is there some builder pattern/crate i can use to reduce this?)
            sender_query: sq,
            receiver_rows: rr,

            client: t_api::establish_http_client(),

            input: Input::default(),
            mode: AppMode::default(),
            table_state: TableState::default(),
            scroll_state: ScrollbarState::default(),
            shows: Vec::new(),

            show_details: None,
            show_seasons: vec![],
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        // WIP implementation of query from our data rows
        // (right now, just pull everything on boot)
        if self.shows.len() == 0 {
            match self.sender_query.send(String::from("spurious")) {
                Ok(_) => {}
                Err(SendError(err)) => {
                    info!("discon {}", err);
                    self.quit();
                }
            }

            match self.receiver_rows.recv() {
                Ok(items) => {
                    self.scroll_state = self.scroll_state.content_length(items.len() as u16);
                    self.shows = items;

                    if self.mode == AppMode::Initializing {
                        self.mode = AppMode::MainView;
                    }
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
            Some(i) => std::cmp::min(i + step, self.shows.len() - 1),
            None => 0,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i as u16);
    }

    pub fn prev(&mut self, step: usize) {
        let i = match self.table_state.selected() {
            Some(i) => std::cmp::max(i as i32 - step as i32, 0) as usize,
            None => self.shows.len() - 1,
        };
        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i as u16);
    }

    /// Cycle watch status of a currently-selected show in main window
    pub fn toggle_watch_status(&mut self) {
        if let Some(i) = self.table_state.selected() {
            let show = &mut self.shows[i];
            info!("Currently selected show: {:?}", show);

            show.user_status = match show.user_status {
                UserStatus::Todo => UserStatus::Watched,
                UserStatus::Watched => UserStatus::Unwatched,
                UserStatus::Unwatched => UserStatus::Todo,
            };

            // update db
            t_db::update_show(show);
        }
    }

    pub async fn enter_show_details(&mut self) {
        if self.mode == AppMode::MainView && let Some(i) = self.table_state.selected() {
            let show = &self.shows[i];
            let (show_details, season_details) =
                t_api::query_detailed(&self.client, &show.imdb_id).await;

            // TODO - when i have these, add them to the db
            // t_db::update_show_info(&ctx ...);

            self.show_details = Some(show_details);
            self.show_seasons = season_details;

            self.mode = AppMode::SeasonView;
        }
    }
}
