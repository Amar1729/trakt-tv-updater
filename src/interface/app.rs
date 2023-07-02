use ratatui::widgets::{Table, TableState};

pub struct Show<'a> {
    pub trakt_id: i64,
    pub imdb_id: &'a str,
    pub original_name: &'a str,
    pub start_year: i64,
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,

    // pub tabs: TabsState<'a>,
    // pub servers: Vec<Server<'a>>,
    // pub enhanced_graphics: bool,
    pub state: TableState,
    pub items: Vec<Show<'a>>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, enhanced_graphics: bool) -> App<'a> {
        App {
            title,
            should_quit: false,
            // enhanced_graphics,
            state: TableState::default().with_selected(Some(0)),
            items: vec![
                Show {
                    trakt_id: 1,
                    imdb_id: "tt01",
                    original_name: "Some show",
                    start_year: 2000,
                },
                Show {
                    trakt_id: 2,
                    imdb_id: "tt02",
                    original_name: "New show",
                    start_year: 1998,
                },
            ],
        }
    }

    // pub fn on_right(&mut self) {
    //     self.tabs.next();
    // }

    // pub fn on_left(&mut self) {
    //     self.tabs.previous();
    // }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {}
}
