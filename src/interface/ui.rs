use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
    Frame,
};

use crate::interface::app::{App, InputMode};

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

    let chunks_textinput = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(10), Constraint::Min(1)].as_ref())
        .split(outer[0]);

    // placeholder for now:
    // this will be text input for searching for shows
    let (msg, style) = match app.mode {
        InputMode::Normal => (vec![Span::raw("Search ")], Style::default()),
        InputMode::Editing => (
            vec![Span::styled(
                "Search > ",
                Style::default().add_modifier(Modifier::BOLD),
            )],
            Style::default(),
        ),
    };

    let mut text = Text::from(Line::from(msg));
    text.patch_style(style);
    let help_msg = Paragraph::new(text);

    frame.render_widget(help_msg, chunks_textinput[0]);

    {
        let width = chunks_textinput[1].width.max(3) - 3;
        let scroll = app.input.visual_scroll(width as usize);
        let input = Paragraph::new(app.input.value())
            .style(match app.mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .scroll((0, scroll as u16))
            .block(Block::default().borders(Borders::NONE));

        frame.render_widget(input, chunks_textinput[1]);

        match app.mode {
            InputMode::Normal => {}
            InputMode::Editing => frame.set_cursor(
                chunks_textinput[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16,
                chunks_textinput[1].y,
            ),
        }
    }

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
