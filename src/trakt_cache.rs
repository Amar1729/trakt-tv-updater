/// Deal with Trakt API:
/// - query trakt API
/// - cache info from trakt in local db
/// This file sort of has a lot of functionality, i may have to split it up later.
use crate::models::{TraktShow, UserStatus};
use crate::schema::trakt_shows;

use chrono::prelude::*;
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
struct ApiIDs {
    trakt: u32,
    slug: String,
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

pub fn establish_ctx() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    SqliteConnection::establish(&database_url).unwrap_or_else(|err| {
        info!("{}", err);
        panic!();
    })
}

/// count the rows in the db
/// (this should be fast - am i doing this inefficiently?)
pub fn count_trakt_db(ctx: &mut SqliteConnection) -> usize {
    use self::trakt_shows::dsl::*;

    let rows = trakt_shows
        .select(TraktShow::as_select())
        .load_iter(ctx)
        .unwrap();

    rows.into_iter().count()
}

/// Return all rows in the db that are not marked as unwatched, and don't have release in the future
pub fn load_filtered_shows(ctx: &mut SqliteConnection) -> Vec<TraktShow> {
    let cap_year = Utc::now().year() + 1;

    trakt_shows::table
        .order_by(trakt_shows::release_year)
        .filter(trakt_shows::release_year.le(cap_year))
        .filter(trakt_shows::user_status.ne(UserStatus::Unwatched))
        .select(TraktShow::as_returning())
        .load(ctx)
        .unwrap()
}

/// update the status of a show **in the DB**
pub fn update_show(show: &TraktShow) {
    use self::trakt_shows::dsl::*;

    let mut ctx = establish_ctx();

    match diesel::insert_into(trakt_shows)
        .values(show)
        .returning(TraktShow::as_returning())
        .on_conflict(imdb_id)
        .do_update()
        .set(user_status.eq(&show.user_status))
        .execute(&mut ctx)
    {
        Ok(_) => {
            info!("Updated row: {}", &show.imdb_id);
        }
        Err(err) => {
            info!("panik on update: {}", err);
        }
    }
}

/// Overwrites (or fills) db with the rows parsed from an IMDB data dump.
pub fn prefill_db_from_imdb(ctx: &mut SqliteConnection, rows: &Vec<TraktShow>) {
    info!("Filling db...");

    use self::trakt_shows::dsl::*;

    for row in rows {
        match diesel::insert_into(trakt_shows)
            .values(row)
            .returning(TraktShow::as_returning())
            .on_conflict(imdb_id)
            .do_update()
            // update the values that might be updated in a new data dump
            .set((
                release_year.eq(&row.release_year),
                no_seasons.eq(&row.no_seasons),
                no_episodes.eq(&row.no_episodes),
            ))
            .execute(ctx)
        {
            Ok(_c) => {
                // can i count only which rows were updated?
                info!("Inserted row: {}", &row.imdb_id);
            }
            Err(err) => {
                // TODO: if this errs, should bubble up and quit app?
                info!("Failed db insert: {}", err);
                panic!();
            }
        }
    }

    info!("Inserted/Updated {} rows.", rows.len());
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

/// Gets show results from searching trakt for an IMDB id (this should be unambiguous)
pub async fn query_trakt_api(client: &reqwest::Client, imdb_id: u32) -> Vec<ApiShow> {
    let search_url = format!("{}/search/imdb/{}", TRAKT_URL, imdb_id);

    let response = client.get(search_url).send().await.unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            // should actually match properly on text not coming back?
            let text = response.text().await.unwrap();

            match serde_json::from_str::<ApiResponse>(&text) {
                Ok(response) => {
                    return response
                        .results
                        .into_iter()
                        .map(|api_match| api_match.show)
                        .collect()
                }
                Err(other) => panic!("missed {:?}", other),
            }
        }
        reqwest::StatusCode::UNAUTHORIZED => unimplemented!(),
        _ => panic!("panic"),
    }
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
