use super::schema::{episodes, seasons, trakt_shows};

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

macro_rules! ratatui_line {
    ($line:expr) => {
        vec![ratatui::text::Line::from($line.to_string())]
    };
}

#[derive(Clone, Debug, PartialEq, diesel_derive_enum::DbEnum)]
pub enum UserStatusSeason {
    Unfilled,
    OnRelease,
    OtherDate,
}

impl From<UserStatusSeason> for ratatui::text::Text<'_> {
    fn from(value: UserStatusSeason) -> Self {
        match value {
            UserStatusSeason::Unfilled => Self {
                lines: ratatui_line!("UNFILLED"),
            },
            UserStatusSeason::OnRelease => Self {
                lines: ratatui_line!("ON_RELEASE"),
            },
            UserStatusSeason::OtherDate => Self {
                lines: ratatui_line!("OTHER_DATE"),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, diesel_derive_enum::DbEnum)]
pub enum UserStatusEpisode {
    Unwatched,
    Watched,
}

impl From<UserStatusEpisode> for ratatui::text::Text<'_> {
    fn from(value: UserStatusEpisode) -> Self {
        match value {
            UserStatusEpisode::Unwatched => Self {
                lines: ratatui_line!("UNWATCHED"),
            },
            UserStatusEpisode::Watched => Self {
                lines: ratatui_line!("WATCHED"),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, diesel_derive_enum::DbEnum)]
pub enum UserStatusShow {
    Unwatched,
    Todo,
    Watched,
}

impl From<UserStatusShow> for ratatui::text::Text<'_> {
    fn from(value: UserStatusShow) -> Self {
        match value {
            UserStatusShow::Unwatched => Self {
                lines: ratatui_line!("UNWATCHED"),
            },
            UserStatusShow::Todo => Self {
                lines: ratatui_line!("TODO"),
            },
            UserStatusShow::Watched => Self {
                lines: ratatui_line!("WATCHED"),
            },
        }
    }
}

/// Track the specific datetime at which a user watched an episode.
// #[derive(Clone, Debug, PartialEq, diesel_derive_enum::DbEnum)]
#[derive(Clone, Debug, PartialEq)]
pub struct UserDateTimeStatus {
    pub imdb_id: String,
    pub trakt_id: i32,
    pub watched: chrono::DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = trakt_shows)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TraktShow {
    pub imdb_id: String,
    pub trakt_id: Option<i32>,
    pub primary_title: String,
    pub original_title: String,
    pub country: Option<String>,
    pub release_year: Option<i32>,
    pub network: Option<String>,
    pub no_seasons: Option<i32>,
    pub no_episodes: Option<i32>,
    pub user_status: UserStatusShow,
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = seasons)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TraktSeason {
    pub id: i32,
    pub show_id: i32,
    pub season_number: i32,
    pub user_status: UserStatusSeason,
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = episodes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TraktEpisode {
    pub id: i32,
    pub show_id: i32,
    pub season_number: i32,
    pub episode_number: i32,
    pub title: String,
    pub first_aired: Option<NaiveDateTime>,
    pub watched_at: Option<NaiveDateTime>,
    pub user_status: UserStatusEpisode,
}
