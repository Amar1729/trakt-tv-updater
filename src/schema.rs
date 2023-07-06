// @generated automatically by Diesel CLI.

diesel::table! {
    trakt_shows (imdb_id) {
        imdb_id -> Text,
        trakt_id -> Nullable<Integer>,
        primary_title -> Text,
        original_title -> Text,
        country -> Nullable<Text>,
        release_year -> Nullable<Integer>,
        network -> Nullable<Text>,
        no_seasons -> Nullable<Integer>,
        no_episodes -> Nullable<Integer>,
    }
}
