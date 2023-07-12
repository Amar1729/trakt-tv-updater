use crate::{
    models::{TraktShow, UserStatus},
    sources::DataManager,
    trakt::{t_api, t_db},
};
use log::*;
use ratatui::widgets::{ScrollbarState, TableState};
use reqwest::Client;
use tui_input::Input;

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
    #[allow(dead_code)]
    HelpWindow,
    /// Detailed view of specific season
    SeasonView,
    // Detailed view of a specific episode
    // not sure about this one yet
    // EpisodeView,
}

/// inner struct for detailed show views.
#[derive(Debug, Default)]
pub struct AppShowView {
    pub show_details: Option<t_api::ApiShowDetails>,
    pub seasons: Vec<t_api::ApiSeasonDetails>,

    pub season_table_state: TableState,
    // unimpl'd yet...
    // pub episodes: Vec<>,
    // pub episode_table_state: TableState,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub data_manager: DataManager,

    /// for querying trakt
    pub client: Client,

    /// local cache of imdb + trakt data
    pub cache: t_db::Database,

    /// ui+handling changes based on the app's current view
    pub mode: AppMode,

    /// used in main view
    pub input: Input,
    pub table_state: TableState,
    pub scroll_state: ScrollbarState,
    pub shows: Vec<TraktShow>,

    // used in season view
    pub show_view: AppShowView,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> eyre::Result<Self> {
        // when a new app is created, begin a bg data manager task
        // this task will receive a string query, and send back a TraktShow vec
        let data_manager = DataManager::init().await?;

        Ok(App {
            running: true,
            data_manager,

            client: t_api::establish_http_client(),
            cache: t_db::Database::connect().await?,

            input: Input::default(),
            mode: AppMode::default(),
            table_state: TableState::default(),
            scroll_state: ScrollbarState::default(),
            shows: Vec::new(),

            show_view: AppShowView::default(),
        })
    }

    /// Handles the tick event of the terminal.
    pub async fn tick(&mut self) -> eyre::Result<()> {
        // WIP implementation of query from our data rows
        // (right now, just pull everything on boot)
        if self.shows.is_empty() {
            let items = self
                .data_manager
                .query(String::from("spurious"))
                .await
                .ok_or_else(|| {
                    error!("data manager thread panicked!");
                    eyre::eyre!("data manager thread panicked!")
                })?;

            self.scroll_state = self.scroll_state.content_length(items.len() as u16);
            self.shows = items;

            if self.mode == AppMode::Initializing {
                self.mode = AppMode::MainView;
            }
        }

        Ok(())
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

    pub fn season_next(&mut self, step: usize) {
        let max = self.show_view.seasons.len() - 1;
        let i = match self.show_view.season_table_state.selected() {
            Some(i) => std::cmp::min(i + step, max),
            None => 0,
        };
        self.show_view.season_table_state.select(Some(i));
    }

    pub fn season_prev(&mut self, step: usize) {
        let i = match self.show_view.season_table_state.selected() {
            Some(i) => std::cmp::max(i as i32 - step as i32, 0) as usize,
            None => 0,
        };
        self.show_view.season_table_state.select(Some(i));
    }

    /// Cycle watch status of a currently-selected show in main window
    pub async fn toggle_watch_status(&mut self) -> eyre::Result<()> {
        if let Some(i) = self.table_state.selected() {
            let show = &mut self.shows[i];
            info!("Currently selected show: {:?}", show);

            show.user_status = match show.user_status {
                UserStatus::Todo => UserStatus::Watched,
                UserStatus::Watched => UserStatus::Unwatched,
                UserStatus::Unwatched => UserStatus::Todo,
            };

            // update db
            t_db::Database::connect()
                .await?
                .update_show(show.clone())
                .await?;
        }

        Ok(())
    }

    pub async fn enter_show_details(&mut self) -> eyre::Result<()> {
        if self.mode == AppMode::MainView && let Some(i) = self.table_state.selected() {
            let show = &self.shows[i];
            match t_api::query_detailed(&self.client, &show.imdb_id).await {
                Ok((show_details, season_details)) => {
                    // TODO - when i have these, add them to the db
                    // t_db::update_show_info(&ctx ...);

                    self.show_view.show_details = Some(show_details);
                    if !season_details.is_empty() {
                        self.show_view.season_table_state.select(Some(0));
                    }
                    self.show_view.seasons = season_details;

                    self.mode = AppMode::SeasonView;
                }
                Err(other) => {
                    error!("error querying show details: {}", other);
                    self.quit();
                    eyre::bail!(other);
                }
            }
        }

        Ok(())
    }
}
