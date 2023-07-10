use crossbeam::channel::{Receiver, RecvError, Sender};
use diesel::SqliteConnection;
use log::*;

use crate::models::TraktShow;
use crate::trakt::{t_api, t_db};

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
fn load_combined_data_sources(ctx: &mut SqliteConnection) -> Vec<TraktShow> {
    // load all shows, and fill db if db is empty
    // let items = imdb_reader::load_show_vec();

    let row_count = t_db::count_trakt_db(ctx);
    info!("row count: {}", row_count);

    if row_count < 100 {
        // if we dont have many rows in db (clean env or devel), load from imdb data
        let items = imdb_reader::load_show_vec();
        // TODO: put this on a thread, once i figure out borrowing?
        t_db::prefill_db_from_imdb(ctx, &items);

        items
    } else {
        // query everything from db
        t_db::load_filtered_shows(ctx)
    }
}

/// A reader that reads data and then waits to respond to queries.
pub fn data_manager(sender: Sender<Vec<TraktShow>>, receiver: Receiver<String>) {
    std::thread::spawn(move || {
        let mut ctx = t_db::establish_ctx();
        let items = load_combined_data_sources(&mut ctx);

        loop {
            match receiver.recv() {
                Ok(_query) => {
                    // TODO: can i pass this as a ref instead of cloning?
                    // would that even be better?
                    sender.send(items.clone()).unwrap();
                }
                // happens when channel is empty + becomes disconnected
                // i think this only happens when user shuts down app
                Err(RecvError) => {}
            }
        }
    });
}
