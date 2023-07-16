#![feature(let_chains)]
#![feature(type_alias_impl_trait)]

mod interface;
mod models;
mod schema;
mod sources;
mod trakt;

use log::*;
use simplelog::*;
use std::fs::File;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    // init logging
    WriteLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::default()
            .set_location_level(simplelog::LevelFilter::Info)
            .build(),
        File::create("trakt_updater.log").unwrap(),
    )
    .unwrap();

    interface::run().await
}
