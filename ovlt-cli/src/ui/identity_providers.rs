use ratatui::{layout::Rect, Frame};

use crate::{app::App, components::table::StatefulTable};

pub fn render(frame: &mut Frame, app: &App, area: Rect, table: &mut StatefulTable) {
    let tenant_name = app.active_tenant_name().unwrap_or("?");
    let title = format!(" Identity Providers — {tenant_name} ");

    let rows: Vec<Vec<String>> = app
        .identity_providers
        .iter()
        .map(|idp| {
            vec![
                idp.provider.clone(),
                idp.client_id[..idp.client_id.len().min(16)].to_string()
                    + if idp.client_id.len() > 16 { "…" } else { "" },
                idp.redirect_url.clone(),
                idp.scopes.join(" "),
                if idp.enabled {
                    "enabled".into()
                } else {
                    "disabled".into()
                },
            ]
        })
        .collect();

    table.select(app.idp_selected);
    table.render(
        frame,
        area,
        &title,
        &["Provider", "Client ID", "Redirect URL", "Scopes", "Status"],
        rows,
    );
}
