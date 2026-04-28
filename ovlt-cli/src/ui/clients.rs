use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_name = app.active_tenant_name().unwrap_or("?");
    let title = format!(" Clients — {tenant_name} ");

    let rows: Vec<Vec<String>> = app
        .clients
        .iter()
        .map(|c| {
            let client_type = if c.grant_types.iter().any(|g| g == "client_credentials") {
                "M2M"
            } else if !c.is_confidential {
                "SPA"
            } else {
                "Conf"
            };
            let ttl = match (c.access_token_ttl_minutes, c.refresh_token_ttl_days) {
                (Some(a), Some(r)) => format!("{a}m/{r}d"),
                (Some(a), None) => format!("{a}m/—"),
                (None, Some(r)) => format!("—/{r}d"),
                (None, None) => "—".into(),
            };
            vec![
                c.name.clone(),
                client_type.into(),
                c.client_id[..8].to_string() + "…",
                c.scopes.join(" "),
                ttl,
                if c.is_active {
                    "✓".into()
                } else {
                    "✗".into()
                },
            ]
        })
        .collect();

    table.select(app.client_selected);
    table.render(
        frame,
        area,
        &title,
        &["Name", "Type", "Client ID", "Scopes", "TTL", "Active"],
        rows,
    );
}
