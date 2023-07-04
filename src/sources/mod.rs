use crossbeam::channel::{Receiver, Sender};

use crate::models::TraktShow;

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
        let items = imdb_reader::get_show_vec();

        // TODO: use proper logging
        // TODO: actually use incoming query
        // TODO: figure out whatever's the right way to handle these errors
        //   (i think i want to close the app if the receiver shuts down?)
        loop {
            let _query = receiver.recv().unwrap();
            sender.send(items.clone()).unwrap();
        }
    });
}
