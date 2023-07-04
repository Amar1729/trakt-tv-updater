use super::schema::trakt_shows;

use diesel::prelude::*;

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = trakt_shows)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TraktShow {
    pub imdb_id: String,
    pub trakt_id: Option<i32>,
    pub tmdb_id: Option<i32>,
    pub primary_title: String,
    pub original_title: String,
    pub country: Option<String>,
    pub release_year: Option<i32>,
    pub network: Option<String>,
    pub no_seasons: Option<i32>,
    pub no_episodes: Option<i32>,
}
