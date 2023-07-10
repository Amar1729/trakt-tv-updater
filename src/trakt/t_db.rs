use crate::models::{TraktShow, UserStatus};
use crate::schema::trakt_shows;

use chrono::prelude::*;
use log::*;
use std::env;

use diesel::prelude::*;
use dotenvy::dotenv;

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
pub fn update_show(show: &TraktShow) -> eyre::Result<()> {
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
            Ok(())
        }
        Err(err) => {
            info!("panik on update: {}", err);
            Err(eyre::eyre!(err))
        }
    }
}

/// Overwrites (or fills) db with the rows parsed from an IMDB data dump.
pub fn prefill_db_from_imdb(ctx: &mut SqliteConnection, rows: &Vec<TraktShow>) -> eyre::Result<()> {
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
                return Err(eyre::eyre!(err));
            }
        }
    }

    info!("Inserted/Updated {} rows.", rows.len());

    Ok(())
}
