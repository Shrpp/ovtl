use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Center a rect of given width/height within parent.
pub fn centered_rect(width: u16, height: u16, parent: Rect) -> Rect {
    let x = parent.x + parent.width.saturating_sub(width) / 2;
    let y = parent.y + parent.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(parent.width),
        height: height.min(parent.height),
    }
}

pub fn render_confirm(frame: &mut Frame, label: &str) {
    let area = centered_rect(50, 7, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = vec![
        Line::from(""),
        Line::from(format!("Delete: {label}")),
        Line::from(""),
        Line::from(vec![
            Span::styled("[y]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Yes   "),
            Span::styled("[n]", Style::default().fg(Color::Green)),
            Span::raw(" No"),
        ]),
    ];

    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}

pub fn render_error(frame: &mut Frame, msg: &str) {
    let area = centered_rect(60, 7, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let para = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}

pub fn render_secret(frame: &mut Frame, client_id: &str, secret: &str) {
    let area = centered_rect(70, 9, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Client Secret (shown once) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let text = vec![
        Line::from(""),
        Line::from(format!("Client ID: {client_id}")),
        Line::from(""),
        Line::from(Span::styled(secret, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("[Enter] Close", Style::default().fg(Color::DarkGray))),
    ];

    let para = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(para, area);
}

pub fn render_edit_user(frame: &mut Frame, email: &str, is_active: bool) {
    let area = centered_rect(56, 10, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Edit User ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect { x: area.x + 2, y: area.y + 1, width: area.width - 4, height: area.height - 2 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Email   ", Style::default().fg(Color::DarkGray)),
            Span::raw(email),
        ])),
        chunks[0],
    );

    let status_color = if is_active { Color::Green } else { Color::Red };
    let status_text = if is_active { "● active" } else { "○ inactive" };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("Status  ", Style::default().fg(Color::DarkGray)),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("   Space → toggle", Style::default().fg(Color::DarkGray)),
        ])),
        chunks[1],
    );

    let hints = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Cyan)),
        Span::styled(" Save   ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::Cyan)),
        Span::styled(" Cancel", Style::default().fg(Color::DarkGray)),
    ]);
    frame.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[4]);
}

/// Render a simple form modal with labelled fields.
/// `fields`: list of (label, value) pairs.
/// `active_field`: index of the currently focused input.
pub fn render_form(
    frame: &mut Frame,
    title: &str,
    fields: &[(&str, &str)],
    active_field: usize,
) {
    let height = (fields.len() as u16) * 3 + 5;
    let area = centered_rect(60, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width - 4,
        height: area.height - 2,
    };

    let mut constraints = vec![];
    for _ in fields {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Min(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    for (i, (label, value)) in fields.iter().enumerate() {
        let is_active = i == active_field;
        let border_style = if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let input_block = Block::default()
            .title(*label)
            .borders(Borders::ALL)
            .border_style(border_style);

        let display = if is_active {
            format!("{value}█")
        } else {
            value.to_string()
        };

        let para = Paragraph::new(display).block(input_block);
        frame.render_widget(para, chunks[i]);
    }

    let hint_area = chunks[fields.len()];
    let hint = Paragraph::new("[Tab] Next field   [Enter] Submit   [Esc] Cancel")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, hint_area);
}
