/// Deal with Trakt API:
/// - query trakt API
/// - cache info from trakt in local db
use crate::models::TraktShow;
use crate::schema::trakt_shows;

use log::*;
use std::{env, thread, time};

use diesel::prelude::*;
use dotenvy::dotenv;
use governor::{Quota, RateLimiter};
use nonzero_ext::*;
use reqwest::header;
use serde::{Deserialize, Serialize};

const APP_USER_AGENT: &str = "Trakt TV Selector";
// 1/sec -> 300 per 5min
const RATE_LIMIT: u32 = 3u32;
const TIME_STEP: time::Duration = time::Duration::from_millis(100);

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
        println!("{}", t.original_title);
    }
}

// probably should be renamed / reworked
// i expect that querying alone will be a pretty large switch on all the ways we will filter from
// the db
pub fn query_trakt_db(ctx: &mut SqliteConnection, trakt_id: i32) -> Option<TraktShow> {
    trakt_shows::table
        .filter(trakt_shows::trakt_id.eq(trakt_id))
        .select(TraktShow::as_select())
        .first(ctx)
        .optional()
        .unwrap()
}

pub fn write_trakt_db(ctx: &mut SqliteConnection, show: ApiShow) -> TraktShow {
    // should be a sql query that does insert or instead?
    //
    // insert into trakt_shows (
    //     id, imdb_id, name, release_year
    // )
    //     select 11, 'tt0011', 'Fake Show', 0
    // where not EXISTS (
    //     select * from trakt_shows where id = 11)
    if let Some(local_result) = query_trakt_db(ctx, show.ids.trakt as i32) {
        println!("returning local result: {:?}", local_result);
        return local_result;
    }

    // TODO: some results from trakt API will give back null IMDB ID.
    // it can't be the primary key if our initial data pull is from trakt.
    // however, if our initial data is from IMDB, then primary key as imdb id makes sense.
    let new_show = TraktShow {
        trakt_id: Some(show.ids.trakt as i32),
        imdb_id: show.ids.imdb.unwrap(),
        primary_title: show.title.clone(),
        original_title: show.title,
        country: None,
        release_year: match show.year {
            Some(y) => Some(y as i32),
            None => None,
        },
        network: None,
        no_seasons: None,
        no_episodes: None,
        user_status: "TODO".to_string(),
    };

    diesel::insert_into(trakt_shows::table)
        .values(&new_show)
        .returning(TraktShow::as_returning())
        .get_result(ctx)
        .expect("err saving new trakt_show")
}

pub async fn query_trakt_api(client: &reqwest::Client, imdb_id: u32) -> Vec<ApiShow> {
    let search_url = format!("http://127.0.0.1:8080/search/imdb/{}", imdb_id);

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
            }
            Err(err) => {
                // TODO: if this errs, should bubble up and quit app?
                info!("Failed db insert: {}", err);
                panic!();
            }
        }
    }
}

pub async fn fill_trakt_db_from_imdb(ctx: &mut SqliteConnection, imdb_id: u32) {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert("Authorization", header::HeaderValue::from_static("token"));

    // TOOD: i guess we should create this client outside somewhere?
    // and then reuse it for any request the app makes when we query for a show
    // if we move over to making all requests from the app, we may able to remove the rate limiting
    // (on our side) since that might be too slow to exceed limits
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .default_headers(headers)
        .build()
        .unwrap();

    let lim = RateLimiter::direct(Quota::per_second(nonzero!(RATE_LIMIT)));

        // TODO - change this function to just hydrate_trakt
        // we'll hydrate from IMDB first (and maybe remove tmdb entirely?)
        // if let Some(imdb_rows) = trakt_shows::table
        //     .filter(trakt_shows::imdb_id.eq())

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
