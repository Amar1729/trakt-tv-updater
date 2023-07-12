use log::*;

use crate::models::TraktShow;
use crate::trakt::t_db;

pub mod imdb_reader;

// i have an idea once the db reading is more fleshed out:
// 1. chunk DB reads so they happen faster
// 2. let our manager tell the app when it has updates (e.g. send an int)
// 3. whenever app.ticks and/or queries, it will see there are new updates
// 4. receive updates?
// not sure about this workflow, maybe i could just create a bounded(0) channel
// over which to send the rows.
// either way, maybe an enum of possible send values (u32 / vec) could be helpful
// pub enum DataResult {}

/// Load all shows from imdb data dump and db.
/// TODO: for now, assume that first startup fills all rows into db.
/// Eventually, we may have to update this to update db on startup from a new data dump.
async fn load_combined_data_sources(db: &mut t_db::Database) -> eyre::Result<Vec<TraktShow>> {
    // load all shows, and fill db if db is empty
    // let items = imdb_reader::load_show_vec();

    let row_count = db.count_shows().await;
    info!("row count: {}", row_count);

    if row_count < 100 {
        // if we dont have many rows in db (clean env or devel), load from imdb data
        let items = imdb_reader::load_show_vec();
        // TODO: put this on a thread, once i figure out borrowing?
        db.prefill_from_imdb(items.clone()).await.map(|()| items)
    } else {
        // query everything from db
        Ok(db.filtered_shows().await)
    }
}

#[derive(Debug)]
pub struct DataManager {
    items: Vec<TraktShow>,
}

impl DataManager {
    pub async fn init() -> eyre::Result<DataManager> {
        let mut db = t_db::Database::connect().await?;
        let items = load_combined_data_sources(&mut db).await?;

        Ok(DataManager { items })
    }

    /// Returns `None` if the servicing thread has died.
    pub async fn query(&self, _q: String) -> Option<Vec<TraktShow>> {
        Some(self.items.clone())
    }
}
