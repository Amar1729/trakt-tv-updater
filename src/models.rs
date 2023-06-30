use super::schema::trakt_shows;

use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = trakt_shows)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TraktShow {
    pub id: i32,
    pub tmdb_id: i32,
    pub name: String,
    pub country: Option<String>,
    pub release_year: Option<i32>,
    pub network: Option<String>,
    pub no_seasons: Option<i32>,
    pub no_episodes: Option<i32>,
}

// #[derive(Insertable)]
// #[diesel(table_name = trakt_shows)]
// pub struct NewTraktShow<'a> {
//     pub id: i32,
//     pub tmdb_id: i32,
//     pub name: &'a str,
//     pub country: Option<&'a str>,
//     pub release_year: Option<i32>,
//     pub network: Option<&'a str>,
//     pub no_seasons: Option<i32>,
//     pub no_episodes: Option<i32>,
// }
