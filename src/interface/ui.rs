use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
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
            Cell::from(show.release_year.unwrap().to_string()),
        ])
    });

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

/// Renders the user interface widgets.
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    match app.mode {
        AppMode::Initializing => unimplemented!(),
        AppMode::MainView => render_main_view(app, frame),
        AppMode::Querying => render_main_view(app, frame),
        AppMode::HelpWindow => unimplemented!(),
        AppMode::SeasonView => unimplemented!(),
    }
}
