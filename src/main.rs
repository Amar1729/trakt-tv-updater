mod interface;
mod models;
mod schema;
mod sources;
mod trakt_cache;

use crossbeam::channel::unbounded;
use diesel::prelude::*;
use dotenvy::dotenv;
use log::*;
use simplelog::*;
use std::{env, fs::File};

fn establish_ctx() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    SqliteConnection::establish(&database_url).unwrap_or_else(|_| panic!("error"))
}

#[tokio::main]
async fn main() {
    // let ctx = &mut establish_ctx();
    // let tmdb_ids = tmdb_reader::read_tv_series()
    //     .unwrap()
    //     .into_iter()
    //     .map(|show| show.id)
    //     .collect();

    // TODO: i'll toss this initial read into a thread (probably will remove tmdb backend)
    // let items = sources::imdb_reader::get_show_vec();

    // init logging
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("trakt_updater.log").unwrap(),
    )
    .unwrap();

    let (sender_query, receiver_query) = unbounded();
    let (sender_rows, receiver_rows) = unbounded();

    sources::data_manager(sender_rows, receiver_query);
    let _ = interface::run(sender_query, receiver_rows);
}
