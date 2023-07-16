CREATE TABLE trakt_shows (
    imdb_id VARCHAR PRIMARY KEY NOT NULL,
    trakt_id INTEGER,
    primary_title VARCHAR NOT NULL,
    original_title VARCHAR NOT NULL,
    country VARCHAR,
    release_year INTEGER,
    network VARCHAR,
    no_seasons INTEGER,
    no_episodes INTEGER,
    overview TEXT,
    user_status TEXT CHECK(user_status IN ('unwatched', 'todo', 'watched')) NOT NULL
);

CREATE TABLE seasons (
    -- trakt_id
    id INTEGER PRIMARY KEY NOT NULL,
    show_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    user_status TEXT CHECK(user_status IN ('unfilled', 'on_release', 'other_date')) NOT NULL,

    FOREIGN KEY(show_id) REFERENCES trakt_shows(trakt_id)
);

CREATE TABLE episodes (
    -- trakt_id
    id INTEGER PRIMARY KEY NOT NULL,
    show_id INTEGER NOT NULL,
    season_number INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    title TEXT NOT NULL,
    --   overview TEXT,
    first_aired DATETIME,

    -- datetime for when user watched this episode
    watched_at DATETIME,

    user_status TEXT CHECK(user_status IN ('unwatched', 'watched')) NOT NULL,

    FOREIGN KEY(show_id) REFERENCES seasons(id)
);