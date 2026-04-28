use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_name = app.active_tenant_name().unwrap_or("?");
    let title = format!(" Users — {tenant_name} ");

    let rows: Vec<Vec<String>> = app
        .users
        .iter()
        .map(|u| {
            vec![
                u.id[..8].to_string() + "…",
                u.email.clone(),
                if u.is_active {
                    "active".into()
                } else {
                    "inactive".into()
                },
                if u.mfa_enabled {
                    "✓".into()
                } else {
                    "✗".into()
                },
                u.created_at[..10].to_string(),
            ]
        })
        .collect();

    table.select(app.user_selected);
    table.render(
        frame,
        area,
        &title,
        &["ID", "Email", "Status", "MFA", "Created"],
        rows,
    );
}
