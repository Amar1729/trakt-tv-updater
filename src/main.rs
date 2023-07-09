#![feature(let_chains)]

mod interface;
mod models;
mod schema;
mod sources;
mod trakt_cache;

use log::*;
use simplelog::*;
use std::fs::File;

#[tokio::main]
async fn main() {
    // init logging
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("trakt_updater.log").unwrap(),
    )
    .unwrap();

    let _ = interface::run().await;
}
