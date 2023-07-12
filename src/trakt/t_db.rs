use crate::models::{TraktShow, UserStatus};
use crate::schema::trakt_shows;

use chrono::prelude::*;
use eyre::Context;
use log::*;
use std::env;
use std::sync::Arc;

use diesel::prelude::*;
use dotenvy::dotenv;

/// A handle to the database of show data. Async methods for queries and updates
/// are provided, synchronization happens internally.
#[derive(Clone)]
pub struct Database {
    conn: Conn,
}

type Conn = Arc<tokio::sync::Mutex<SqliteConnection>>;

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Database { ... }")
    }
}

impl Database {
    pub fn connect_sync() -> eyre::Result<Database> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        Ok(Database {
            conn: Arc::new(tokio::sync::Mutex::new(SqliteConnection::establish(
                &database_url,
            )?)),
        })
    }

    pub async fn connect() -> eyre::Result<Database> {
        tokio::task::spawn_blocking(Self::connect_sync)
            .await
            .unwrap()
    }

    /// count the rows in the db
    /// (this should be fast - am i doing this inefficiently?)
    pub async fn count_shows(&self) -> usize {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || Self::count_shows_impl(conn))
            .await
            .unwrap()
    }

    /// Return all rows in the db that are not marked as unwatched, and don't have release in the future
    pub async fn filtered_shows(&self) -> Vec<TraktShow> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || Self::filtered_shows_impl(conn))
            .await
            .unwrap()
    }

    /// update the status of a show **in the DB**
    pub async fn update_show(&self, show: TraktShow) -> eyre::Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || Self::update_show_impl(conn, &show))
            .await
            .unwrap()
    }

    /// Overwrites (or fills) db with the rows parsed from an IMDB data dump.
    pub async fn prefill_from_imdb(&self, rows: Vec<TraktShow>) -> eyre::Result<()> {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || Self::prefill_from_imdb_impl(conn, &rows))
            .await
            .unwrap()
    }

    fn count_shows_impl(conn: Conn) -> usize {
        use self::trakt_shows::dsl::*;

        let mut conn = conn.blocking_lock();
        let rows = trakt_shows
            .select(TraktShow::as_select())
            .load_iter(&mut *conn)
            .unwrap();

        rows.into_iter().count()
    }

    fn filtered_shows_impl(conn: Conn) -> Vec<TraktShow> {
        let cap_year = Utc::now().year() + 1;

        let mut conn = conn.blocking_lock();
        trakt_shows::table
            .order_by(trakt_shows::release_year)
            .filter(trakt_shows::release_year.le(cap_year))
            .filter(trakt_shows::user_status.ne(UserStatus::Unwatched))
            .select(TraktShow::as_returning())
            .load(&mut *conn)
            .unwrap()
    }

    fn update_show_impl(conn: Conn, show: &TraktShow) -> eyre::Result<()> {
        use self::trakt_shows::dsl::*;

        let mut conn = conn.blocking_lock();
        diesel::insert_into(trakt_shows)
            .values(show)
            .on_conflict(imdb_id)
            .do_update()
            .set(user_status.eq(&show.user_status))
            .execute(&mut *conn)
            .map(|_| ())
            .wrap_err("could not update show in db")?;

        info!("Updated row: {}", &show.imdb_id);
        Ok(())
    }

    fn prefill_from_imdb_impl(conn: Conn, rows: &Vec<TraktShow>) -> eyre::Result<()> {
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
                .execute(&mut *conn.blocking_lock())
                .map(|_| ())
                .wrap_err("could not insert show")?;

            info!("Inserted row: {}", &row.imdb_id);
        }

        info!("Inserted/Updated {} rows.", rows.len());

        Ok(())
    }
}
