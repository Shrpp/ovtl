use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let rows: Vec<Vec<String>> = app
        .tenants
        .iter()
        .map(|t| {
            vec![
                t.id[..8].to_string() + "…",
                t.name.clone(),
                t.slug.clone(),
                t.plan.clone(),
                t.created_at[..10].to_string(),
            ]
        })
        .collect();

    table.select(app.tenant_selected);
    table.render(
        frame,
        area,
        " Tenants ",
        &["ID", "Name", "Slug", "Plan", "Created"],
        rows,
    );
}

pub fn keybinds() -> Vec<(&'static str, &'static str)> {
    vec![
        ("↑↓", "Navigate"),
        ("n", "New"),
        ("r", "Refresh"),
        ("→", "Clients"),
        ("q", "Quit"),
    ]
}
