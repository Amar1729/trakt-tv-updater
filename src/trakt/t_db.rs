use crate::models::{TraktSeason, TraktShow, UserStatusSeason, UserStatusShow};
use crate::schema::{seasons, trakt_shows};
use crate::trakt::t_api::ApiSeasonDetails;

use std::env;
use std::future::Future;
use std::sync::Arc;

use chrono::prelude::*;
use diesel::prelude::*;
use dotenvy::dotenv;
use eyre::Context;
use futures::FutureExt;
use log::*;
use tokio::sync::Mutex;

/// The cache database's interface. This is a trait to allow ease of testing.
pub trait Database {
    type Fut<T>: Future<Output = T>;

    /// Count shows stored in the database.
    fn count_shows(&self) -> Self::Fut<usize>;

    /// Get all shows that are unwatched and are released.
    fn filtered_shows(&self) -> Self::Fut<Vec<TraktShow>>;

    /// Update the database status of a show.
    fn update_show(&self, show: TraktShow) -> Self::Fut<eyre::Result<()>>;

    fn update_season(&self, season: TraktSeason) -> Self::Fut<eyre::Result<()>>;

    fn update_show_with_seasons(
        &self,
        show: &TraktShow,
        api_seasons: &[ApiSeasonDetails],
    ) -> Self::Fut<eyre::Result<Vec<TraktSeason>>>;

    /// Fill database with shows loaded from the IMDB dump.
    fn prefill_from_imdb(&self, rows: Vec<TraktShow>) -> Self::Fut<eyre::Result<()>>;
}

/// Handle to sqlite-backed persistent database. Provides an async interface
/// with synchronization handled inside.
#[derive(Clone)]
pub struct PersistentDb {
    conn: Conn,
}

type Conn = Arc<Mutex<SqliteConnection>>;

impl std::fmt::Debug for PersistentDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PersistentDb { ... }")
    }
}

// Name the anonymous future (with unstable type_alias_impl_trait feature) so it
// may be referenced in a trait impl.
pub type PersistentDbFuture<T> = impl Future<Output = T>;

impl Database for PersistentDb {
    type Fut<T> = PersistentDbFuture<T>;

    fn count_shows(&self) -> Self::Fut<usize> {
        self.on_blocking_task(Self::count_shows_impl)
    }

    fn filtered_shows(&self) -> PersistentDbFuture<Vec<TraktShow>> {
        self.on_blocking_task(Self::filtered_shows_impl)
    }

    fn update_show(&self, show: TraktShow) -> PersistentDbFuture<eyre::Result<()>> {
        self.on_blocking_task(move |conn| Self::update_show_impl(conn, &show))
    }

    fn update_season(&self, season: TraktSeason) -> Self::Fut<eyre::Result<()>> {
        self.on_blocking_task(move |conn| Self::update_season_impl(conn, &season))
    }

    fn update_show_with_seasons(
        &self,
        show: &TraktShow,
        api_seasons: &[ApiSeasonDetails],
    ) -> Self::Fut<eyre::Result<Vec<TraktSeason>>> {
        let trakt_seasons: Vec<_> = api_seasons
            .iter()
            .map(|s| TraktSeason {
                id: s.ids.trakt as i32,
                title: s.title.clone(),
                first_aired: Some(s.first_aired.naive_utc()),
                show_id: show.trakt_id.unwrap() as i32,
                season_number: s.number as i32,
                episode_count: s.episode_count as i32,
                user_status: UserStatusSeason::Unfilled,
            })
            .collect();

        self.on_blocking_task(move |conn| Self::update_show_with_seasons_impl(conn, trakt_seasons))
    }

    fn prefill_from_imdb(&self, rows: Vec<TraktShow>) -> PersistentDbFuture<eyre::Result<()>> {
        self.on_blocking_task(move |conn| Self::prefill_from_imdb_impl(conn, &rows))
    }
}

impl PersistentDb {
    pub fn connect_sync() -> eyre::Result<PersistentDb> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        Ok(PersistentDb {
            conn: Arc::new(Mutex::new(SqliteConnection::establish(&database_url)?)),
        })
    }

    pub async fn connect() -> eyre::Result<PersistentDb> {
        tokio::task::spawn_blocking(Self::connect_sync)
            .await
            .unwrap()
    }

    fn on_blocking_task<T, F>(&self, f: F) -> PersistentDbFuture<T>
    where
        F: 'static + Send + FnOnce(&mut SqliteConnection) -> T,
        T: 'static + Send,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = conn.blocking_lock();
            f(&mut *conn)
        })
        .map(Result::unwrap)
    }

    fn count_shows_impl(conn: &mut SqliteConnection) -> usize {
        use self::trakt_shows::dsl::*;

        let rows: i64 = trakt_shows.count().get_result(conn).unwrap();
        rows.try_into().unwrap()
    }

    fn filtered_shows_impl(conn: &mut SqliteConnection) -> Vec<TraktShow> {
        let cap_year = Utc::now().year() + 1;

        trakt_shows::table
            .order_by(trakt_shows::release_year)
            .filter(trakt_shows::release_year.le(cap_year))
            .filter(trakt_shows::user_status.ne(UserStatusShow::Unwatched))
            .select(TraktShow::as_returning())
            .load(conn)
            .unwrap()
    }

    fn update_show_impl(conn: &mut SqliteConnection, show: &TraktShow) -> eyre::Result<()> {
        use self::trakt_shows::dsl::*;

        diesel::insert_into(trakt_shows)
            .values(show)
            .on_conflict(imdb_id)
            .do_update()
            .set((
                trakt_id.eq(&show.trakt_id),
                user_status.eq(&show.user_status),
                overview.eq(&overview),
            ))
            .execute(conn)
            .map(|_| ())
            .wrap_err("could not update show in db")?;

        info!("Updated row: {}", &show.imdb_id);
        Ok(())
    }

    pub fn update_season_impl(
        conn: &mut SqliteConnection,
        season: &TraktSeason,
    ) -> eyre::Result<()> {
        use self::seasons::dsl::*;

        let updated_season: TraktSeason = diesel::update(seasons.filter(id.eq(season.id)))
            .set((
                season_number.eq(&season.season_number),
                episode_count.eq(&season.episode_count),
                user_status.eq(&season.user_status),
            ))
            .get_result(conn)?;

        info!("Updated to season: {:?}", updated_season);

        // TODO: update the episodes in this season, if necessary
        // (if user selects ON_RELEASE for this season)

        Ok(())
    }

    fn update_show_with_seasons_impl(
        conn: &mut SqliteConnection,
        trakt_seasons: Vec<TraktSeason>,
    ) -> eyre::Result<Vec<TraktSeason>> {
        use self::seasons::dsl::*;

        for season in trakt_seasons.iter() {
            diesel::insert_into(seasons)
                .values(season)
                .on_conflict(id)
                .do_update()
                .set(season_number.eq(season.season_number))
                .execute(conn)
                .wrap_err("failed db insert")?;
        }

        Ok(trakt_seasons)
    }

    fn prefill_from_imdb_impl(
        conn: &mut SqliteConnection,
        rows: &Vec<TraktShow>,
    ) -> eyre::Result<()> {
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
                .execute(conn)
                .map(|_| ())
                .wrap_err("could not insert show")?;

            info!("Inserted row: {}", &row.imdb_id);
        }

        info!("Inserted/Updated {} rows.", rows.len());

        Ok(())
    }
}
