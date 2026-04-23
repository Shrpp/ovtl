use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let status = match &app.health_status {
        Some(s) => Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::raw(s.clone()),
        ]),
        None => Line::from(Span::styled(
            "● Connecting…",
            Style::default().fg(Color::Yellow),
        )),
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Server Status",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        status,
        Line::from(""),
        Line::from(Span::raw(format!("Endpoint: {}", app.client.base_url))),
    ];

    let block = Block::default().borders(Borders::ALL).title(" Health ");
    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(para, area);
}

pub fn keybinds() -> Vec<(&'static str, &'static str)> {
    vec![("r", "Refresh"), ("q", "Quit")]
}
