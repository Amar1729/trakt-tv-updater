use crossbeam::channel::{Receiver, RecvError, Sender};

use crate::models::TraktShow;
use crate::trakt_cache;

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

/// A reader that reads data and then waits to respond to queries.
pub fn data_manager(sender: Sender<Vec<TraktShow>>, receiver: Receiver<String>) {
    std::thread::spawn(move || {
        let mut ctx = trakt_cache::establish_ctx();
        let items = imdb_reader::load_show_vec();

        loop {
            match receiver.recv() {
                Ok(_query) => {
                    // TODO: can i pass this as a ref instead of cloning?
                    sender.send(items.clone()).unwrap();

                    // TODO: only do this once, on starting this function?
                    trakt_cache::prefill_db_from_imdb(&mut ctx, &items);
                }
                // happens when channel is empty + becomes disconnected
                // i think this only happens when user shuts down app
                Err(RecvError) => {}
            }
        }
    });
}
