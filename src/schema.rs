// @generated automatically by Diesel CLI.
// with UserStatusMapping, this has been manually updated.
// (provided by diesel_derive_enum)

diesel::table! {
    use diesel::sql_types::{
        Text,
        Nullable,
        Integer,
    };
    use crate::models::UserStatusMapping;
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
        user_status -> UserStatusMapping,
    }
}
