CREATE TABLE trakt_shows (
    imdb_id VARCHAR PRIMARY KEY NOT NULL,
    trakt_id INTEGER,
    primary_title VARCHAR NOT NULL,
    original_title VARCHAR NOT NULL,
    country VARCHAR,
    release_year INTEGER,
    network VARCHAR,
    no_seasons INTEGER,
    no_episodes INTEGER
)
