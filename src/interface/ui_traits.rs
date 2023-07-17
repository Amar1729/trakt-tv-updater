use ratatui::{
    text::{Line, Text},
    widgets::Cell,
};

use crate::models::{TraktSeason, TraktShow};

// implementation of From trait for TraktSeason to a ratatui table Row
impl From<&TraktSeason> for ratatui::widgets::Row<'_> {
    fn from(season: &TraktSeason) -> Self {
        ratatui::widgets::Row::new(vec![
            Cell::from(season.season_number.to_string()),
            Cell::from(season.title.clone()),
            Cell::from(season.episode_count.to_string()),
            // Cell::from(season.first_aired.format("%Y-%m-%d").to_string()),
            Cell::from(
                season
                    .first_aired
                    .unwrap_or_default()
                    // possible TODO: datetime conversion here?
                    .format("%Y-%m-%d UTC")
                    .to_string(),
            ),
            // map season watch status to a ratatui Text
            Cell::from(Text::from(season.user_status.clone())),
        ])
    }
}

// implementation of From trait for TraktShow to ratatui Text
impl From<&TraktShow> for ratatui::text::Text<'_> {
    fn from(show: &TraktShow) -> Self {
        Self {
            lines: vec![
                Line::default(),
                Line::from(format!(
                    "Release Year: {}",
                    show.release_year.unwrap_or_default()
                )),
                Line::from(format!(
                    "Network: {}",
                    show.network.clone().unwrap_or_default()
                )),
                Line::from(format!(
                    "{} seasons, {} episodes",
                    show.no_seasons.unwrap_or(0),
                    show.no_episodes.unwrap_or(0)
                )),
                Line::default(),
                Line::from(show.overview.clone().unwrap_or_default()),
            ],
        }
    }
}

// implementation of From trait for TraktShow to ratatui table Row
impl From<&TraktShow> for ratatui::widgets::Row<'_> {
    fn from(show: &TraktShow) -> Self {
        ratatui::widgets::Row::new(vec![
            Cell::from(show.imdb_id.to_string()),
            Cell::from(show.original_title.clone()),
            Cell::from(match show.release_year {
                Some(yy) => yy.to_string(),
                None => "<unreleased>".to_string(),
            }),
            Cell::from(show.user_status.clone()),
        ])
    }
}
