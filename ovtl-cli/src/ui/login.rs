use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, AppMode};

pub fn render(frame: &mut Frame, app: &App) {
    let AppMode::Login {
        email,
        password,
        field,
        error,
    } = &app.mode
    else {
        return;
    };

    let size = frame.area();

    let box_w: u16 = 52;
    let box_h: u16 = 16;
    let area = Rect {
        x: size.x + size.width.saturating_sub(box_w) / 2,
        y: size.y + size.height.saturating_sub(box_h) / 2,
        width: box_w.min(size.width),
        height: box_h.min(size.height),
    };

    frame.render_widget(Clear, area);

    let border_block = Block::default()
        .title(" OVTL Admin ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(border_block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // subtitle
            Constraint::Length(1), // spacer
            Constraint::Length(3), // email
            Constraint::Length(3), // password
            Constraint::Length(1), // spacer
            Constraint::Length(1), // error
            Constraint::Min(1),    // hints
        ])
        .split(inner);

    let subtitle = Paragraph::new("Sign in to continue")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[0]);

    let email_active = *field == 0;
    let border_style = |active: bool| {
        if active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        }
    };

    let email_val = if email_active {
        format!("{email}█")
    } else {
        email.clone()
    };
    let email_widget = Paragraph::new(email_val).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Email")
            .border_style(border_style(email_active)),
    );
    frame.render_widget(email_widget, chunks[2]);

    let pass_active = *field == 1;
    let masked = "*".repeat(password.len());
    let pass_val = if pass_active {
        format!("{masked}█")
    } else {
        masked
    };
    let pass_widget = Paragraph::new(pass_val).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Password")
            .border_style(border_style(pass_active)),
    );
    frame.render_widget(pass_widget, chunks[3]);

    if let Some(err) = error {
        let err_widget = Paragraph::new(Span::styled(
            err.as_str(),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        frame.render_widget(err_widget, chunks[5]);
    }

    let hints = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::styled(" Next   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Login   ", Style::default().fg(Color::DarkGray)),
        Span::styled("q", Style::default().fg(Color::Cyan)),
        Span::styled(" Quit", Style::default().fg(Color::DarkGray)),
    ]);
    let hints_widget = Paragraph::new(hints).alignment(Alignment::Center);
    frame.render_widget(hints_widget, chunks[6]);
}
