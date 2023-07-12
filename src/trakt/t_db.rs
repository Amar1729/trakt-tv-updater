use crate::models::{TraktShow, UserStatus};
use crate::schema::trakt_shows;

use chrono::prelude::*;
use eyre::Context;
use log::*;
use std::env;

use diesel::prelude::*;
use dotenvy::dotenv;

pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    pub fn connect() -> eyre::Result<Database> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        Ok(Database {
            conn: SqliteConnection::establish(&database_url)?,
        })
    }

    /// count the rows in the db
    /// (this should be fast - am i doing this inefficiently?)
    pub fn count_shows(&mut self) -> usize {
        use self::trakt_shows::dsl::*;

        let rows = trakt_shows
            .select(TraktShow::as_select())
            .load_iter(&mut self.conn)
            .unwrap();

        rows.into_iter().count()
    }

    /// Return all rows in the db that are not marked as unwatched, and don't have release in the future
    pub fn filtered_shows(&mut self) -> Vec<TraktShow> {
        let cap_year = Utc::now().year() + 1;

        trakt_shows::table
            .order_by(trakt_shows::release_year)
            .filter(trakt_shows::release_year.le(cap_year))
            .filter(trakt_shows::user_status.ne(UserStatus::Unwatched))
            .select(TraktShow::as_returning())
            .load(&mut self.conn)
            .unwrap()
    }

    /// update the status of a show **in the DB**
    pub fn update_show(&mut self, show: &TraktShow) -> eyre::Result<()> {
        use self::trakt_shows::dsl::*;

        diesel::insert_into(trakt_shows)
            .values(show)
            .on_conflict(imdb_id)
            .do_update()
            .set(user_status.eq(&show.user_status))
            .execute(&mut self.conn)
            .map(|_| ())
            .wrap_err("could not update show in db")?;

        info!("Updated row: {}", &show.imdb_id);
        Ok(())
    }

    /// Overwrites (or fills) db with the rows parsed from an IMDB data dump.
    pub fn prefill_from_imdb(&mut self, rows: &Vec<TraktShow>) -> eyre::Result<()> {
        info!("Filling db...");

        use self::trakt_shows::dsl::*;

        for row in rows {
            diesel::insert_into(trakt_shows)
                .values(row)
                .on_conflict(imdb_id)
                .do_update()
                // update the values that might be updated in a new data dump
                .set((
                    release_year.eq(&row.release_year),
                    no_seasons.eq(&row.no_seasons),
                    no_episodes.eq(&row.no_episodes),
                ))
                .execute(&mut self.conn)
                .map(|_| ())
                .wrap_err("could not insert show")?;

            info!("Inserted row: {}", &row.imdb_id);
        }

        info!("Inserted/Updated {} rows.", rows.len());

        Ok(())
    }
}
