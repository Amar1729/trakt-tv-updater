use super::schema::trakt_shows;

use diesel::prelude::*;

macro_rules! ratatui_line {
    ($line:expr) => {
        vec![ratatui::text::Line::from($line.to_string())]
    };
}

#[derive(Clone, Debug, PartialEq, diesel_derive_enum::DbEnum)]
pub enum UserStatus {
    Unwatched,
    Todo,
    Watched,
}

impl From<UserStatus> for ratatui::text::Text<'_> {
    fn from(value: UserStatus) -> Self {
        match value {
            UserStatus::Unwatched => Self {
                lines: ratatui_line!("UNWATCHED"),
            },
            UserStatus::Todo => Self {
                lines: ratatui_line!("TODO"),
            },
            UserStatus::Watched => Self {
                lines: ratatui_line!("WATCHED"),
            },
        }
    }
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
    pub user_status: UserStatus,
}
