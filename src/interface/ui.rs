use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        Table, Wrap,
    },
    Frame,
};

use crate::interface::app::{App, AppMode};

/// Render text input widget for querying shows
fn render_input_area<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(1)].as_ref())
        .split(area);

    let (msg, style) = match app.mode {
        AppMode::MainView => (vec![Span::raw("Search ")], Style::default()),
        AppMode::Querying => (
            vec![Span::styled(
                "Search > ",
                Style::default().add_modifier(Modifier::REVERSED),
            )],
            Style::default(),
        ),
        _ => panic!(),
    };

    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_msg = Paragraph::new(text);

    frame.render_widget(help_msg, chunks[0]);

    let width = chunks[1].width.max(3) - 3;
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.mode {
            AppMode::MainView => Style::default(),
            AppMode::Querying => Style::default().fg(Color::Yellow),
            _ => panic!(),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::NONE));

    frame.render_widget(input, chunks[1]);

    match app.mode {
        AppMode::MainView => {}
        AppMode::Querying => frame.set_cursor(
            chunks[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16,
            chunks[1].y,
        ),
        _ => panic!(),
    }
}

/// Render table of all tv shows
fn render_shows_table<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(area);

    let rows = app.shows.iter().map(|show| {
        Row::new(vec![
            Cell::from(show.imdb_id.as_str()),
            Cell::from(show.original_title.clone()),
            Cell::from(match show.release_year {
                Some(yy) => yy.to_string(),
                None => "<unreleased>".to_string(),
            }),
            Cell::from(show.user_status.clone()),
        ])
    });

    frame.render_stateful_widget(
        Table::new(rows)
            .header(
                Row::new(vec![
                    "imdb_id",
                    "original_name",
                    "start_year",
                    "user_status",
                ])
                .style(Style::default().fg(Color::Yellow)),
            )
            .block(Block::default().title("Shows").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">> ")
            .widths(&[
                Constraint::Length(12),
                Constraint::Length(35),
                Constraint::Length(13),
                Constraint::Length(12),
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
        chunks[1],
        &mut app.scroll_state,
    )
}

/// renders main view (includes search bar)
fn render_main_view<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)].as_ref())
        .split(frame.size());

    render_input_area(app, frame, outer[0]);
    render_shows_table(app, frame, outer[1]);
}

fn initalize_app<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(12),
                Constraint::Min(0),
                Constraint::Percentage(12),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                // empty space
                Constraint::Length(2),
                // "Initializing..."
                Constraint::Length(4),
                // progress bar
                Constraint::Length(4),
                // rest of space
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    let widget = Paragraph::new("Initializing ...")
        .style(Style::default())
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray)),
        );

    frame.render_widget(widget, chunks[1]);

    // doesn't actually track progress of initialization yet.
    // i load all items at once on startup.
    let progress = Gauge::default().percent(0).block(
        Block::default()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray)),
    );

    frame.render_widget(progress, chunks[2])
}

/// Render details for a TV season.
fn render_season_view<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    if let Some(s_info) = &app.show_view.show_details {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(frame.size());

        let text = Text::from(vec![
            Line::default(),
            Line::from(format!("Release Year: {}", s_info.year.to_string())),
            Line::from(format!("Network: {}", s_info.network.as_str())),
            Line::from(format!("Episodes: {}", s_info.aired_episodes)),
            Line::default(),
            Line::from(s_info.overview.as_str()),
        ]);

        let widget = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .style(Style::default())
            .block(
                Block::default()
                    .title(s_info.title.as_str())
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Gray)),
            );

        frame.render_widget(widget, chunks[0]);

        let rows = app.show_view.seasons.iter().map(|season| {
            Row::new(vec![
                Cell::from(season.number.to_string()),
                Cell::from(season.title.to_string()),
                Cell::from(season.episode_count.to_string()),
                Cell::from(season.first_aired.format("%Y-%m-%d").to_string()),
            ])
        });

        // render a stateless season table for now.
        frame.render_widget(
            Table::new(rows)
                .header(
                    Row::new(vec![
                        "season #",
                        "title",
                        "#episodes",
                        "aired",
                        // user_watched?
                    ])
                    .style(Style::default().fg(Color::Yellow)),
                )
                .block(Block::default().title("Seasons").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ")
                .widths(&[
                    Constraint::Length(9),
                    Constraint::Length(20),
                    Constraint::Length(10),
                    Constraint::Length(12),
                ])
                .style(Style::default().fg(Color::Cyan).bg(Color::Black)),
            chunks[1],
        );

        return;
    }

    // can't be called unless there's a row selected in main view.
    unreachable!()
}

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    match app.mode {
        AppMode::Initializing => initalize_app(app, frame),
        AppMode::MainView => render_main_view(app, frame),
        AppMode::Querying => render_main_view(app, frame),
        AppMode::HelpWindow => unimplemented!(),
        AppMode::SeasonView => render_season_view(app, frame),
        // uh oh, i'm worried i'll need an episode view as well?
    }
}
