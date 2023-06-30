use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize)]
pub struct TmdbTvShow {
    pub id: u32,
    pub original_name: String,
    pub popularity: f32,
}

pub fn read_tv_series() -> Result<Vec<TmdbTvShow>> {
    let file_path = "tv_short.json";

    let mut file = File::open(file_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    Ok(contents
        .lines()
        .map(|line| serde_json::from_str::<TmdbTvShow>(&line).unwrap())
        .collect())
}
