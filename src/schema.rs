// @generated automatically by Diesel CLI.

diesel::table! {
    tmdb_progress (id) {
        id -> Integer,
        queried -> Bool,
    }
}

diesel::table! {
    trakt_shows (imdb_id) {
        imdb_id -> Text,
        trakt_id -> Nullable<Integer>,
        tmdb_id -> Nullable<Integer>,
        primary_title -> Text,
        original_title -> Text,
        country -> Nullable<Text>,
        release_year -> Nullable<Integer>,
        network -> Nullable<Text>,
        no_seasons -> Nullable<Integer>,
        no_episodes -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(tmdb_progress, trakt_shows,);
