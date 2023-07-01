/// Deal with Trakt API:
/// - query trakt API
/// - cache info from trakt in local db
use crate::models::TraktShow;
use crate::schema::trakt_shows;

use std::{thread, time};

use diesel::prelude::*;
use reqwest::header;
use serde::{Deserialize, Serialize};
use nonzero_ext::*;
use governor::{Quota, RateLimiter};

const APP_USER_AGENT: &str = "Trakt TV Selector";
// 1/sec -> 300 per 5min
const RATE_LIMIT: u32 = 3u32;
const TIME_STEP: time::Duration = time::Duration::from_millis(100);

#[derive(Serialize, Deserialize, Debug)]
struct ApiIDs {
    trakt: u32,
    slug: String,
    tvdb: Option<u32>,
    imdb: Option<String>,
    tmdb: u32,
    tvrage: Option<u32>,
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

pub fn read_trakt_db(ctx: &mut SqliteConnection) {
    // TODO: dev function for now to remind me how queries work.
    // i'll redo this once i get a user interface up.
    let results = trakt_shows::table
        .limit(5)
        .order_by(trakt_shows::release_year)
        .filter(trakt_shows::release_year.is_not_null())
        .select(TraktShow::as_select())
        .load(ctx)
        .expect("err");

    println!("displaying: {} ", results.len());
    for t in results {
        println!("{}", t.name);
    }
}

// probably should be renamed / reworked
// i expect that querying alone will be a pretty large switch on all the ways we will filter from
// the db
pub fn query_trakt_db(ctx: &mut SqliteConnection, trakt_id: i32) -> Option<TraktShow> {
    trakt_shows::table
        .filter(trakt_shows::id.eq(trakt_id))
        .select(TraktShow::as_select())
        .first(ctx)
        .optional()
        .unwrap()
}

pub fn write_trakt_db(ctx: &mut SqliteConnection, show: ApiShow) -> TraktShow {
    // should be a sql query that does insert or instead?
    //
    // insert into trakt_shows (
    //     id, tmdb_id, name, release_year
    // )
    //     select 11, 11, 'Fake Show', 0
    // where not EXISTS (
    //     select * from trakt_shows where id = 11)
    if let Some(local_result) = query_trakt_db(ctx, show.ids.trakt as i32) {
        println!("returning local result: {:?}", local_result);
        return local_result;
    }

    let new_show = TraktShow {
        id: show.ids.trakt as i32,
        tmdb_id: show.ids.tmdb as i32,
        imdb_id: show.ids.imdb,
        name: show.title,
        country: None,
        release_year: match show.year {
            Some(y) => Some(y as i32),
            None => None,
        },
        network: None,
        no_seasons: None,
        no_episodes: None,
    };

    diesel::insert_into(trakt_shows::table)
        .values(&new_show)
        .returning(TraktShow::as_returning())
        .get_result(ctx)
        .expect("err saving new trakt_show")
}

pub async fn query_trakt_api(client: &reqwest::Client, tmdb_id: u32) -> Vec<ApiShow> {
    let search_url = format!("http://127.0.0.1:8080/search/{}", tmdb_id);

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
        reqwest::StatusCode::UNAUTHORIZED => {
            unimplemented!()
        }
        _ => {
            panic!("panic")
        }
    }
}

pub async fn hydrate_trakt_from_tmdb(ctx: &mut SqliteConnection, tmdb_ids: Vec<u32>) {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert("Authorization", header::HeaderValue::from_static("token"));

    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()
        .unwrap();

    let lim = RateLimiter::direct(Quota::per_second(nonzero!(RATE_LIMIT)));

    // let mut shows: Vec<TraktShow> = vec![];
    for tmdb_id in tmdb_ids {
        // check if tmdb_id is in local database
        // if tmdb_id is not in our local db, we have to query the API
        if let Some(tmdb_rows) = trakt_shows::table
            .filter(trakt_shows::tmdb_id.eq(tmdb_id as i32))
            .select(TraktShow::as_select())
            .load(ctx)
            .optional()
            .unwrap()
        {
            if tmdb_rows.len() > 0 {
                println!("found tmdb rows: {} {}", tmdb_rows.len(), tmdb_rows[0].name);
                // shows.append(tmdb_rows);
                continue;
            }
        }

        // TODO - change this function to just hydrate_trakt
        // we'll hydrate from IMDB first (and maybe remove tmdb entirely?)
        // if let Some(imdb_rows) = trakt_shows::table
        //     .filter(trakt_shows::imdb_id.eq())

        // until_ready should block until the limiter is ready to submit another job, right?
        // but it doesn't, so instead i'm doing this wacky loop{} construction
        // lim.until_ready().await;
        loop {
            match lim.check() {
                Ok(_) => {
                    for api_show in query_trakt_api(&client, tmdb_id).await {
                        println!("querying...");
                        // shows.push(api_show)
                        write_trakt_db(ctx, api_show);
                    }

                    break;
                },
                Err(_) => {},
            }

            thread::sleep(TIME_STEP);
        }
    }
}

mod tests {
    #[test]
    fn check_rate_limit() {
        use crate::trakt_cache::RATE_LIMIT;
        let total = RATE_LIMIT * 60 * 5;
        assert!(total < 1000u32);
    }
}
