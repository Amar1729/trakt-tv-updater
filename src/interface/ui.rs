use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, Cell,
        Row, Table, Tabs,
    },
    Frame,
};

use crate::interface::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let titles = vec!["tab0", "tab1"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(app.title))
        .highlight_style(Style::default().fg(Color::Yellow))
        .select(0);

    f.render_widget(tabs, chunks[0]);
    draw_second_tab(f, app, chunks[1]);
}

fn draw_second_tab<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let selected_block_style = Style::default().fg(Color::Green);

    let up_style = Style::default().fg(Color::Green);
    let failure_style = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::RAPID_BLINK | Modifier::CROSSED_OUT);

    let rows = app.items.iter().map(|show| {
        Row::new(vec![
            Cell::from(show.imdb_id),
            Cell::from(show.original_name),
            Cell::from(show.start_year.to_string()),
        ])
    });

    let table = Table::new(rows)
        .header(
            Row::new(vec!["imdb_id", "original_name", "start_year"])
                .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().title("Shows").border_style(selected_block_style).borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Length(15),
            Constraint::Length(15),
            Constraint::Length(10),
        ]);

    f.render_stateful_widget(table, area, &mut app.state);
}
