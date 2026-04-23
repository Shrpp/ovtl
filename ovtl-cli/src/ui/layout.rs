use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, Screen};

pub fn split_areas(frame: &Frame, _app: &App) -> (Rect, Rect, Rect) {
    let size = frame.area();
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
        .split(size);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(0)])
        .split(outer[1]);

    (body[1], outer[0], outer[2])
}

pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let connected = app.health_status.as_deref().unwrap_or("connecting…");
    let text = Line::from(vec![
        Span::styled(
            " OVTL Admin ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ "),
        Span::styled(&app.client.base_url, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(connected, Style::default().fg(Color::Green)),
    ]);
    let header = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

pub fn render_nav(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = [Screen::Tenants, Screen::Clients, Screen::Health]
        .into_iter()
        .map(|s| {
            let label = match &s {
                Screen::Tenants => " Tenants",
                Screen::Clients => " Clients",
                Screen::Health => " Health",
            };
            let style = if app.screen == s {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let mut state = ListState::default();
    let selected = match app.screen {
        Screen::Tenants => 0,
        Screen::Clients => 1,
        Screen::Health => 2,
    };
    state.select(Some(selected));

    let nav = List::new(items).block(Block::default().borders(Borders::ALL).title(" Menu "));
    frame.render_stateful_widget(nav, area, &mut state);
}
