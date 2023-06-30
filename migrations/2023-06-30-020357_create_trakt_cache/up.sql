CREATE TABLE trakt_shows (
    id INTEGER PRIMARY KEY NOT NULL,
    tmdb_id INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    country VARCHAR,
    release_year INTEGER,
    network VARCHAR,
    no_seasons INTEGER,
    no_episodes INTEGER
)
