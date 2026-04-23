use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_label = app
        .active_tenant_id
        .as_deref()
        .unwrap_or("(no tenant selected)");

    let title = format!(" Clients — {tenant_label} ");

    let rows: Vec<Vec<String>> = app
        .clients
        .iter()
        .map(|c| {
            vec![
                c.id[..8].to_string() + "…",
                c.name.clone(),
                c.client_id[..8].to_string() + "…",
                c.scopes.join(" "),
                if c.is_active { "yes".into() } else { "no".into() },
            ]
        })
        .collect();

    table.select(app.client_selected);
    table.render(
        frame,
        area,
        &title,
        &["ID", "Name", "Client ID", "Scopes", "Active"],
        rows,
    );
}

pub fn keybinds() -> Vec<(&'static str, &'static str)> {
    vec![
        ("↑↓", "Navigate"),
        ("n", "New"),
        ("d", "Deactivate"),
        ("r", "Refresh"),
        ("←", "Tenants"),
        ("q", "Quit"),
    ]
}
