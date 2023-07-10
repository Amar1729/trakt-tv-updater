use chrono::{DateTime, Utc};
use log::*;
use std::{env, thread, time};

use diesel::prelude::*;
use dotenvy::dotenv;
use governor::{Quota, RateLimiter};
use nonzero_ext::*;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const APP_USER_AGENT: &str = "Trakt TV Selector";
// 1/sec -> 300 per 5min
const RATE_LIMIT: u32 = 3u32;
const TIME_STEP: time::Duration = time::Duration::from_millis(100);

// for testing, i've copied over several JSON responses and host them locally.
const TRAKT_URL: &str = "http://127.0.0.1:8080";
// const TRAKT_URL: &str = "https://api.trakt.tv/api";
// const TRAKT_URL: &str = "https://api-staging.trakt.tv/api";

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiIDs {
    trakt: u32,
    slug: Option<String>,
    imdb: Option<String>,
    // skipping other unimportant IDs
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiShow {
    title: String,
    year: Option<u32>,
    ids: ApiIDs,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiMatch {
    #[serde(rename = "type")]
    pub _type: String,
    pub score: u32,
    pub show: ApiShow,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub results: Vec<ApiMatch>,
}

// shows/<id>?extended=full
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiShowDetails {
    pub title: String,
    pub year: u32,
    pub ids: ApiIDs,
    pub overview: String,
    pub first_aired: DateTime<Utc>,
    pub network: String,
    pub country: String,
    // pub language: String,
    pub aired_episodes: u32,
}

// shows/<id>/seasons?extended=full
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiSeasonDetails {
    pub number: usize,
    pub ids: ApiIDs,
    pub episode_count: usize,
    pub title: String,
    pub first_aired: DateTime<Utc>,
    pub overview: Option<String>,
    pub network: String,
}

/// Creates a single HTTP client to use for trakt.tv requests
pub fn establish_http_client() -> Client {
    // TODO: anyhow/error handle this (as part of app startup?)
    dotenv().ok();

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        header::HeaderValue::from_static(APP_USER_AGENT),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert("trakt-api-version", header::HeaderValue::from_static("2"));

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set.");
    headers.insert(
        "trakt-api-key",
        header::HeaderValue::from_str(&client_id).unwrap(),
    );

    // TODO - don't need auth for searching shows.
    // i'll continue using the unauth'd API for now
    // will need to figure out an oauth flow once i get to updating values
    // let bearer = format!("Bearer {}", env::var("OAUTH_TOKEN").expect("OAUTH_TOKEN must be set."));
    // headers.insert(
    //     "Authorization",
    //     header::HeaderValue::from_str(&bearer).unwrap(),
    // );

    // requests from within a CLI app probably happen too slowly to hit rate limit,
    // so we might just let the app manage itself?
    reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()
        .unwrap()
}

async fn query_show_info(client: &reqwest::Client, imdb_id: &String) -> ApiShowDetails {
    let search_url = format!("{}/shows/{}?extended=full", TRAKT_URL, imdb_id);
    let response = client.get(search_url).send().await.unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            let text = response.text().await.unwrap();

            match serde_json::from_str::<ApiShowDetails>(&text) {
                Ok(response) => response,
                Err(other) => {
                    info!("Failed to parse: {}", other);
                    panic!();
                }
            }
        }
        reqwest::StatusCode::UNAUTHORIZED => unimplemented!(),
        _ => {
            info!("Failed to request");
            panic!();
        }
    }
}

async fn query_season_info(client: &reqwest::Client, imdb_id: &String) -> Vec<ApiSeasonDetails> {
    let search_url = format!("{}/shows/{}/seasons?extended=full", TRAKT_URL, imdb_id);
    let response = client.get(search_url).send().await.unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            let text = response.text().await.unwrap();

            match serde_json::from_str::<Vec<ApiSeasonDetails>>(&text) {
                Ok(response) => response,
                Err(other) => {
                    info!("Failed to parse: {}", other);
                    panic!();
                }
            }
        }
        reqwest::StatusCode::UNAUTHORIZED => unimplemented!(),
        _ => {
            info!("Failed to request");
            panic!();
        }
    }
}

/// Gets detailed show results from searching trakt for an IMDB id (this should be unambiguous)
/// Does two API calls: one for the show info, one for season info
pub async fn query_detailed(
    client: &reqwest::Client,
    imdb_id: &String,
) -> (ApiShowDetails, Vec<ApiSeasonDetails>) {
    (
        query_show_info(client, &imdb_id).await,
        query_season_info(client, &imdb_id).await,
    )
}

pub async fn fill_trakt_db_from_imdb(ctx: &mut SqliteConnection, imdb_id: u32) {
    let lim = RateLimiter::direct(Quota::per_second(nonzero!(RATE_LIMIT)));

    // until_ready should block until the limiter is ready to submit another job, right?
    // but it doesn't, so instead i'm doing this wacky loop{} construction
    // lim.until_ready().await;
    //
    // keeping this around so i remember how to use it
    loop {
        match lim.check() {
            Ok(_) => {
                // for api_show in query_trakt_api(&client, tmdb_id).await {
                //     println!("querying...");
                //     // shows.push(api_show)
                //     write_trakt_db(ctx, api_show);
                // }

                break;
            }
            Err(_) => {}
        }

        thread::sleep(TIME_STEP);
    }

    // this code isn't fully impl'd right now - panic if it's called
    // we'll finalize it once i figure out the right way to poll data from trakt
    // and fill our db
    unimplemented!();
}

mod tests {
    #[test]
    fn check_rate_limit() {
        use super::RATE_LIMIT;
        let total = RATE_LIMIT * 60 * 5;
        assert!(total < 1000u32);
    }
}
