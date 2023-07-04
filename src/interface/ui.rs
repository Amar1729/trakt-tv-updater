use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
    Frame,
};

use crate::interface::app::App;

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(frame.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(outer[1]);

    let rows = app.items.iter().map(|show| {
        Row::new(vec![
            Cell::from(show.imdb_id.as_str()),
            Cell::from(show.original_title.clone()),
            Cell::from(show.release_year.unwrap().to_string()),
        ])
    });

    // placeholder for now:
    // this will be text input for searching for shows
    frame.render_widget(
        Paragraph::new("TODO: Input widget for searching for tv shows")
            .block(Block::default().borders(Borders::NONE)),
        outer[0],
    );

    frame.render_stateful_widget(
        Table::new(rows)
            .header(
                Row::new(vec!["imdb_id", "original_name", "start_year"])
                    .style(Style::default().fg(Color::Yellow)),
            )
            .block(Block::default().title("Shows").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Length(12),
                Constraint::Length(35),
                Constraint::Length(10),
            ])
            .style(Style::default().fg(Color::Cyan).bg(Color::Black)),
        chunks[0],
        &mut app.table_state,
    );

    frame.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        outer[1],
        &mut app.scroll_state,
    )
}
