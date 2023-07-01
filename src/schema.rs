// @generated automatically by Diesel CLI.

diesel::table! {
    tmdb_progress (id) {
        id -> Integer,
        queried -> Bool,
    }
}

diesel::table! {
    trakt_shows (id) {
        id -> Integer,
        tmdb_id -> Integer,
        imdb_id -> Nullable<Text>,
        name -> Text,
        country -> Nullable<Text>,
        release_year -> Nullable<Integer>,
        network -> Nullable<Text>,
        no_seasons -> Nullable<Integer>,
        no_episodes -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    tmdb_progress,
    trakt_shows,
);
