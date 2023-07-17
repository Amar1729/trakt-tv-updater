// @generated automatically by Diesel CLI.

diesel::table! {
    episodes (id) {
        id -> Integer,
        show_id -> Integer,
        season_number -> Integer,
        episode_number -> Integer,
        title -> Text,
        first_aired -> Nullable<Timestamp>,
        watched_at -> Nullable<Timestamp>,
        user_status -> crate::models::UserStatusEpisodeMapping,
    }
}

diesel::table! {
    seasons (id) {
        id -> Integer,
        title -> Text,
        first_aired -> Nullable<Timestamp>,
        show_id -> Integer,
        season_number -> Integer,
        episode_count -> Integer,
        user_status -> crate::models::UserStatusSeasonMapping,
    }
}

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
        overview -> Nullable<Text>,
        user_status -> crate::models::UserStatusShowMapping,
    }
}

diesel::joinable!(episodes -> seasons (show_id));

diesel::allow_tables_to_appear_in_same_query!(episodes, seasons, trakt_shows,);
