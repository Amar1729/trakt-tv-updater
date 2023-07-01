mod models;
mod schema;
mod sources;
mod tmdb_reader;
mod trakt_cache;

use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

fn establish_ctx() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    SqliteConnection::establish(&database_url).unwrap_or_else(|_| panic!("error"))
}

#[tokio::main]
async fn main() {
    let ctx = &mut establish_ctx();
    let tmdb_ids = tmdb_reader::read_tv_series()
        .unwrap()
        .into_iter()
        .map(|show| show.id)
        .collect();

    // TODO: this can run on a background thread
    // then main thread could display TUI (but i'll have to handle querying from sqlite?)
    trakt_cache::hydrate_trakt_from_tmdb(ctx, tmdb_ids).await;
}
