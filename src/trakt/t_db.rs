use crate::models::{TraktSeason, TraktShow, UserStatusSeason, UserStatusShow};
use crate::schema::{seasons, trakt_shows};
use crate::trakt::t_api::ApiSeasonDetails;

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
            .filter(trakt_shows::user_status.ne(UserStatusShow::Unwatched))
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
            .set((
                trakt_id.eq(&show.trakt_id),
                user_status.eq(&show.user_status),
                overview.eq(&overview),
            ))
            .execute(&mut self.conn)
            .map(|_| ())
            .wrap_err("could not update show in db")?;

        info!("Updated row: {}", &show.imdb_id);
        Ok(())
    }

    pub fn update_season(&mut self, season: &TraktSeason) -> eyre::Result<()> {
        use self::seasons::dsl::*;

        let updated_season: TraktSeason = diesel::update(seasons.filter(id.eq(season.id)))
            .set((
                season_number.eq(&season.season_number),
                episode_count.eq(&season.episode_count),
                user_status.eq(&season.user_status),
            ))
            .get_result(&mut self.conn)?;

        info!("Updated to season: {:?}", updated_season);

        // TODO: update the episodes in this season, if necessary
        // (if user selects ON_RELEASE for this season)

        Ok(())
    }

    pub fn update_show_with_seasons(
        &mut self,
        show: &TraktShow,
        api_seasons: &[ApiSeasonDetails],
    ) -> eyre::Result<Vec<TraktSeason>> {
        use self::seasons::dsl::*;

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

        for season in trakt_seasons.iter() {
            diesel::insert_into(seasons)
                .values(season)
                .on_conflict(id)
                .do_update()
                .set(season_number.eq(season.season_number))
                .execute(&mut self.conn)
                .wrap_err("failed db insert")?;
            info!("Updated show season: {} {}", show.imdb_id, season.id);
        }

        Ok(trakt_seasons)
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
