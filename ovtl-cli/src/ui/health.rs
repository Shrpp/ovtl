use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area)[0];

    let (dot_color, dot, status_text) = match (&app.health_status, &app.health_error) {
        (Some(s), _) => (Color::Green, "●", s.as_str()),
        (None, Some(_)) => (Color::Red, "●", "Unreachable"),
        (None, None) => (Color::Yellow, "◌", "Connecting…"),
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Server Health",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("{dot} "),
                Style::default().fg(dot_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(status_text, Style::default().fg(dot_color)),
        ]),
    ];

    if let Some(err) = &app.health_error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {err}"),
            Style::default().fg(Color::Red),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Endpoint  ", Style::default().fg(Color::DarkGray)),
        Span::raw(app.client.base_url.clone()),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(
        Span::styled("Press r to refresh", Style::default().fg(Color::DarkGray)),
    ));

    let block = Block::default().borders(Borders::ALL).title(" Health ");
    let para = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(para, inner);
}

