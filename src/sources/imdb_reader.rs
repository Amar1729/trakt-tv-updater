use std::iter::Iterator;

use log::*;
use serde::{Deserialize, Serialize};

use crate::models::TraktShow;

/// currently unimpl'd: will be used to download IMDB dataset on init
pub fn _download_source() {
    unimplemented!();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImdbShow {
    pub tconst: String,
    pub title_type: String,
    pub primary_title: Option<String>,
    pub original_title: Option<String>,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub start_year: Option<i64>,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub end_year: Option<i64>,
}

/// Read shows from IMDB data dump
fn load_imdb_shows(dump_file_name: &str) -> impl Iterator<Item = ImdbShow> {
    let reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(dump_file_name)
        .unwrap();
    reader
        .into_deserialize()
        .map(|r| r.unwrap())
        .filter(|show: &ImdbShow| ["tvSeries", "tvMiniSeries"].contains(&show.title_type.as_str()))
}

fn load_show_vec_from_source(dump_file_name: &str) -> Vec<TraktShow> {
    info!("Loading from datadump ...");

    // arbitrary limit for testing
    let shows = load_imdb_shows(dump_file_name).take(99);

    info!("Serializing structs...");
    shows
        .map(|show| TraktShow {
            imdb_id: show.tconst,
            trakt_id: None,
            primary_title: show.primary_title.unwrap(),
            original_title: show.original_title.unwrap(),
            release_year: show.start_year.map(|y| y as i32),
            no_seasons: None,
            no_episodes: None,
            country: None,
            network: None,
            user_status: crate::models::UserStatusShow::Todo,
        })
        .collect()
}

pub fn load_show_vec() -> Vec<TraktShow> {
    load_show_vec_from_source(DUMP_FILE_NAME)
}

const DUMP_FILE_NAME: &str = "./title.basics.short.tsv";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_load_sample_file() {
        let _shows = load_show_vec_from_source("title.basics.randomsample");
    }
}
