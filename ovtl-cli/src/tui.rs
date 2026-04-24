use arboard;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::{
    api::ApiError,
    app::{App, AppMode, Focus, Modal, QuickStartState, Tab},
    components::{modal, statusbar, table::StatefulTable},
    events::{poll, AppEvent},
    ui,
};

pub async fn run(mut app: App) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut client_table = StatefulTable::new();
    let mut user_table = StatefulTable::new();

    loop {
        terminal.draw(|frame| {
            if matches!(&app.mode, AppMode::Login { .. }) {
                ui::login::render(frame, &app);
                return;
            }

            let (sidebar, content, header, statusbar_area) = ui::layout::split_areas(frame);
            let (tabs_area, content_body) = ui::layout::split_content(content);

            ui::layout::render_header(frame, &app, header);
            ui::layout::render_tenant_sidebar(frame, &app, sidebar);
            ui::layout::render_tabs(frame, &app, tabs_area);

            match app.tab {
                Tab::Clients => ui::clients::render(frame, &app, content_body, &mut client_table),
                Tab::Users => ui::users::render(frame, &app, content_body, &mut user_table),
                Tab::Health => ui::health::render(frame, &app, content_body),
            }

            let hints: Vec<(&str, &str)> = match app.focus {
                Focus::Sidebar => vec![
                    ("↑↓", "Tenant"),
                    ("→/Enter", "Open"),
                    ("n", "New tenant"),
                    ("r", "Refresh"),
                    ("q", "Quit"),
                ],
                Focus::Content => match app.tab {
                    Tab::Clients => vec![
                        ("←/Esc", "Back"),
                        ("Tab", "Next tab"),
                        ("↑↓", "Navigate"),
                        ("n", "New"),
                        ("e", "Edit"),
                        ("d", "Delete"),
                        ("r", "Refresh"),
                        ("q", "Quit"),
                    ],
                    Tab::Users => vec![
                        ("←/Esc", "Back"),
                        ("Tab", "Next tab"),
                        ("↑↓", "Navigate"),
                        ("n", "New"),
                        ("e", "Edit"),
                        ("d", "Deactivate"),
                        ("r", "Refresh"),
                        ("q", "Quit"),
                    ],
                    Tab::Health => vec![
                        ("←/Esc", "Back"),
                        ("Tab", "Next tab"),
                        ("r", "Refresh"),
                        ("q", "Quit"),
                    ],
                },
            };

            statusbar::render(frame, statusbar_area, &hints, app.status_msg.as_deref());

            match &app.modal.clone() {
                Modal::None => {}
                Modal::ConfirmDelete { id: _, label } => {
                    modal::render_confirm(frame, label);
                }
                Modal::ShowSecret { client_id, secret } => {
                    modal::render_secret(frame, client_id, secret);
                }
                Modal::Error(msg) => {
                    modal::render_error(frame, msg);
                }
                Modal::CreateTenant { name, slug, field } => {
                    modal::render_form(frame, "New Tenant", &[("Name", name), ("Slug", slug)], *field);
                }
                Modal::CreateClient { name, redirect_uri, scopes, field } => {
                    modal::render_form(
                        frame,
                        "New Client",
                        &[("Name", name), ("Redirect URI", redirect_uri), ("Scopes", scopes)],
                        *field,
                    );
                }
                Modal::CreateUser { email, password, field } => {
                    modal::render_form(
                        frame,
                        "New User",
                        &[("Email", email), ("Password", password)],
                        *field,
                    );
                }
                Modal::QuickStart(_) => {
                    ui::quickstart::render(frame, &app);
                }
                Modal::EditClient { name, redirect_uris, scopes, field, .. } => {
                    modal::render_form(
                        frame,
                        "Edit Client",
                        &[("Name", name), ("Redirect URIs", redirect_uris), ("Scopes", scopes)],
                        *field,
                    );
                }
                Modal::EditUser { email, is_active, .. } => {
                    modal::render_edit_user(frame, email, *is_active);
                }
            }
        })?;

        match poll()? {
            Some(AppEvent::Key(key)) => {
                if matches!(&app.mode, AppMode::Login { .. }) {
                    handle_login_key(&mut app, key.code).await;
                } else {
                    handle_key(&mut app, key.code, key.modifiers).await;
                }
                if app.should_quit {
                    break;
                }
            }
            Some(AppEvent::Tick) => {}
            None => break,
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_login_key(app: &mut App, code: KeyCode) {
    let AppMode::Login {
        email,
        password,
        field,
        ..
    } = app.mode.clone()
    else {
        return;
    };

    match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.mode = AppMode::Login {
                email,
                password,
                field: (field + 1) % 2,
                error: None,
            };
        }
        KeyCode::Backspace => {
            let (mut e, mut p) = (email, password);
            if field == 0 {
                e.pop();
            } else {
                p.pop();
            }
            app.mode = AppMode::Login { email: e, password: p, field, error: None };
        }
        KeyCode::Char(c) => {
            let (mut e, mut p) = (email, password);
            if field == 0 {
                e.push(c);
            } else {
                p.push(c);
            }
            app.mode = AppMode::Login { email: e, password: p, field, error: None };
        }
        KeyCode::Enter => {
            if email.is_empty() || password.is_empty() {
                return;
            }
            let client = app.client.clone();
            match client.login(&email, &password).await {
                Ok(token) => {
                    app.client.set_token(token);
                    app.mode = AppMode::Admin;
                    load_tenants(app).await;
                    check_health(app).await;
                    // Auto-open wizard if only the master tenant exists
                    let only_master = app.tenants.len() <= 1
                        && app.tenants.iter().all(|t| t.slug == "master");
                    if only_master {
                        app.modal = Modal::QuickStart(QuickStartState::default());
                    }
                }
                Err(ApiError::Api { status: 401, .. }) => {
                    app.mode = AppMode::Login {
                        email,
                        password,
                        field,
                        error: Some("Invalid credentials".to_string()),
                    };
                }
                Err(e) => {
                    app.mode = AppMode::Login {
                        email,
                        password,
                        field,
                        error: Some(format!("Error: {e}")),
                    };
                }
            }
        }
        _ => {}
    }
}

async fn handle_key(app: &mut App, code: KeyCode, _mods: KeyModifiers) {
    match app.modal.clone() {
        Modal::ConfirmDelete { id, label: _ } => {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => perform_delete(app, id).await,
                _ => app.modal = Modal::None,
            }
            return;
        }
        Modal::ShowSecret { .. } | Modal::Error(_) => {
            app.modal = Modal::None;
            return;
        }
        Modal::CreateTenant { mut name, mut slug, mut field } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                KeyCode::Enter => {
                    if !name.is_empty() && !slug.is_empty() {
                        let n = name.clone();
                        let s = slug.clone();
                        app.modal = Modal::None;
                        perform_create_tenant(app, n, s).await;
                    }
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        name.pop();
                    } else {
                        slug.pop();
                    }
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                KeyCode::Char(c) => {
                    if field == 0 {
                        name.push(c);
                    } else {
                        slug.push(c);
                    }
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                _ => {}
            }
            return;
        }
        Modal::CreateClient { mut name, mut redirect_uri, mut scopes, mut field } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 3;
                    app.modal = Modal::CreateClient { name, redirect_uri, scopes, field };
                }
                KeyCode::Enter => {
                    if !name.is_empty() && !redirect_uri.is_empty() {
                        let n = name.clone();
                        let u = redirect_uri.clone();
                        let sc = scopes.clone();
                        app.modal = Modal::None;
                        perform_create_client(app, n, u, sc).await;
                    }
                }
                KeyCode::Backspace => {
                    match field {
                        0 => {
                            name.pop();
                        }
                        1 => {
                            redirect_uri.pop();
                        }
                        _ => {
                            scopes.pop();
                        }
                    }
                    app.modal = Modal::CreateClient { name, redirect_uri, scopes, field };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => name.push(c),
                        1 => redirect_uri.push(c),
                        _ => scopes.push(c),
                    }
                    app.modal = Modal::CreateClient { name, redirect_uri, scopes, field };
                }
                _ => {}
            }
            return;
        }
        Modal::CreateUser { mut email, mut password, mut field } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 2;
                    app.modal = Modal::CreateUser { email, password, field };
                }
                KeyCode::Enter => {
                    if !email.is_empty() && !password.is_empty() {
                        let e = email.clone();
                        let p = password.clone();
                        app.modal = Modal::None;
                        perform_create_user(app, e, p).await;
                    }
                }
                KeyCode::Backspace => {
                    if field == 0 {
                        email.pop();
                    } else {
                        password.pop();
                    }
                    app.modal = Modal::CreateUser { email, password, field };
                }
                KeyCode::Char(c) => {
                    if field == 0 {
                        email.push(c);
                    } else {
                        password.push(c);
                    }
                    app.modal = Modal::CreateUser { email, password, field };
                }
                _ => {}
            }
            return;
        }
        Modal::EditClient { mut name, mut redirect_uris, mut scopes, mut field, id } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Tab => {
                    field = (field + 1) % 3;
                    app.modal = Modal::EditClient { id, name, redirect_uris, scopes, field };
                }
                KeyCode::Enter => {
                    if !name.is_empty() {
                        let id2 = id.clone();
                        let n = name.clone();
                        let ru = redirect_uris.clone();
                        let sc = scopes.clone();
                        app.modal = Modal::None;
                        perform_edit_client(app, id2, n, ru, sc).await;
                    }
                }
                KeyCode::Backspace => {
                    match field {
                        0 => { name.pop(); }
                        1 => { redirect_uris.pop(); }
                        _ => { scopes.pop(); }
                    }
                    app.modal = Modal::EditClient { id, name, redirect_uris, scopes, field };
                }
                KeyCode::Char(c) => {
                    match field {
                        0 => name.push(c),
                        1 => redirect_uris.push(c),
                        _ => scopes.push(c),
                    }
                    app.modal = Modal::EditClient { id, name, redirect_uris, scopes, field };
                }
                _ => {}
            }
            return;
        }
        Modal::EditUser { id, email, mut is_active } => {
            match code {
                KeyCode::Esc => app.modal = Modal::None,
                KeyCode::Char(' ') => {
                    is_active = !is_active;
                    app.modal = Modal::EditUser { id, email, is_active };
                }
                KeyCode::Enter => {
                    let id2 = id.clone();
                    app.modal = Modal::None;
                    perform_edit_user(app, id2, is_active).await;
                }
                _ => {}
            }
            return;
        }
        Modal::QuickStart(_) => {
            handle_quickstart_key(app, code).await;
            return;
        }
        Modal::None => {}
    }

    if code == KeyCode::Char('q') {
        app.should_quit = true;
        return;
    }

    if code == KeyCode::Char('?') {
        app.modal = Modal::QuickStart(QuickStartState::default());
        return;
    }

    match app.focus {
        Focus::Sidebar => handle_sidebar_key(app, code).await,
        Focus::Content => handle_content_key(app, code).await,
    }
}

async fn handle_sidebar_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up => {
            if app.tenant_selected > 0 {
                app.tenant_selected -= 1;
            }
        }
        KeyCode::Down => {
            if app.tenant_selected + 1 < app.tenants.len() {
                app.tenant_selected += 1;
            }
        }
        KeyCode::Enter | KeyCode::Right => {
            if let Some(t) = app.selected_tenant() {
                let tid = t.id.clone();
                let switching_tenant = app.active_tenant_id.as_deref() != Some(&tid);
                if switching_tenant {
                    app.clients = vec![];
                    app.users = vec![];
                    app.client_selected = 0;
                    app.user_selected = 0;
                }
                app.active_tenant_id = Some(tid.clone());
                app.focus = Focus::Content;
                match app.tab {
                    Tab::Clients => load_clients(app, tid).await,
                    Tab::Users => load_users(app, tid).await,
                    Tab::Health => {}
                }
            }
        }
        KeyCode::Tab => {
            if app.active_tenant_id.is_some() {
                app.focus = Focus::Content;
            }
        }
        KeyCode::Char('n') => {
            app.modal = Modal::CreateTenant {
                name: String::new(),
                slug: String::new(),
                field: 0,
            };
        }
        KeyCode::Char('r') => load_tenants(app).await,
        _ => {}
    }
}

async fn handle_content_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Left => {
            app.focus = Focus::Sidebar;
        }
        KeyCode::Tab => {
            app.tab = match app.tab {
                Tab::Clients => Tab::Users,
                Tab::Users => Tab::Health,
                Tab::Health => Tab::Clients,
            };
            load_current_tab(app).await;
        }
        KeyCode::Up => match app.tab {
            Tab::Clients => {
                if app.client_selected > 0 {
                    app.client_selected -= 1;
                }
            }
            Tab::Users => {
                if app.user_selected > 0 {
                    app.user_selected -= 1;
                }
            }
            Tab::Health => {}
        },
        KeyCode::Down => match app.tab {
            Tab::Clients => {
                if app.client_selected + 1 < app.clients.len() {
                    app.client_selected += 1;
                }
            }
            Tab::Users => {
                if app.user_selected + 1 < app.users.len() {
                    app.user_selected += 1;
                }
            }
            Tab::Health => {}
        },
        KeyCode::Char('n') => match app.tab {
            Tab::Clients => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateClient {
                        name: String::new(),
                        redirect_uri: String::new(),
                        scopes: String::from("openid email profile"),
                        field: 0,
                    };
                }
            }
            Tab::Users => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateUser {
                        email: String::new(),
                        password: String::new(),
                        field: 0,
                    };
                }
            }
            Tab::Health => {}
        },
        KeyCode::Char('e') => match app.tab {
            Tab::Clients => {
                if let Some(c) = app.selected_client() {
                    app.modal = Modal::EditClient {
                        id: c.id.clone(),
                        name: c.name.clone(),
                        redirect_uris: c.redirect_uris.join(", "),
                        scopes: c.scopes.join(" "),
                        field: 0,
                    };
                }
            }
            Tab::Users => {
                if let Some(u) = app.selected_user() {
                    app.modal = Modal::EditUser {
                        id: u.id.clone(),
                        email: u.email.clone(),
                        is_active: u.is_active,
                    };
                }
            }
            Tab::Health => {}
        },
        KeyCode::Char('d') => match app.tab {
            Tab::Clients => {
                if let Some(c) = app.selected_client() {
                    let id = c.id.clone();
                    let label = c.name.clone();
                    app.modal = Modal::ConfirmDelete { id, label };
                }
            }
            Tab::Users => {
                if let Some(u) = app.selected_user() {
                    let id = u.id.clone();
                    let label = u.email.clone();
                    app.modal = Modal::ConfirmDelete { id, label };
                }
            }
            Tab::Health => {}
        },
        KeyCode::Char('r') => load_current_tab(app).await,
        _ => {}
    }
}

async fn load_current_tab(app: &mut App) {
    match app.tab {
        Tab::Clients => {
            if let Some(tid) = app.active_tenant_id.clone() {
                load_clients(app, tid).await;
            }
        }
        Tab::Users => {
            if let Some(tid) = app.active_tenant_id.clone() {
                load_users(app, tid).await;
            }
        }
        Tab::Health => check_health(app).await,
    }
}

async fn load_tenants(app: &mut App) {
    app.tenants_loading = true;
    match app.client.list_tenants().await {
        Ok(list) => {
            app.tenants = list;
            app.tenant_selected = app.tenant_selected.min(app.tenants.len().saturating_sub(1));
            app.clear_status();
        }
        Err(e) => app.set_status(format!("Error: {e}")),
    }
    app.tenants_loading = false;
}

async fn load_clients(app: &mut App, tenant_id: String) {
    app.clients = vec![];
    app.client_selected = 0;
    app.clients_loading = true;
    match app.client.list_clients(&tenant_id).await {
        Ok(list) => {
            app.clients = list;
            app.clear_status();
        }
        Err(e) => app.set_status(format!("Error: {e}")),
    }
    app.clients_loading = false;
}

async fn load_users(app: &mut App, tenant_id: String) {
    app.users = vec![];
    app.user_selected = 0;
    app.users_loading = true;
    match app.client.list_users(&tenant_id).await {
        Ok(list) => {
            app.users = list;
            app.clear_status();
        }
        Err(e) => app.set_status(format!("Error: {e}")),
    }
    app.users_loading = false;
}

async fn check_health(app: &mut App) {
    match app.client.health().await {
        Ok(v) => {
            app.health_status = Some(v["status"].as_str().unwrap_or("ok").to_owned());
            app.health_version = v["version"].as_str().map(|s| s.to_owned());
            app.health_error = None;
        }
        Err(e) => {
            app.health_status = None;
            app.health_version = None;
            app.health_error = Some(e.to_string());
        }
    }
}

async fn perform_create_tenant(app: &mut App, name: String, slug: String) {
    match app.client.create_tenant(&name, &slug).await {
        Ok(_) => {
            app.set_status(format!("Tenant '{name}' created"));
            load_tenants(app).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

async fn perform_create_client(
    app: &mut App,
    name: String,
    redirect_uri: String,
    scopes_str: String,
) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    match app
        .client
        .create_client(&tid, &name, vec![redirect_uri], scopes)
        .await
    {
        Ok(c) => {
            if let Some(secret) = c.client_secret {
                app.modal = Modal::ShowSecret {
                    client_id: c.client_id,
                    secret,
                };
            } else {
                app.set_status(format!("Client '{name}' created"));
            }
            load_clients(app, tid).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

async fn perform_create_user(app: &mut App, email: String, password: String) {
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.client.create_user(&tid, &email, &password).await {
        Ok(_) => {
            app.set_status(format!("User '{email}' created"));
            load_users(app, tid).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

fn copy_to_clipboard(app: &mut App, text: &str, success_msg: &str) {
    match arboard::Clipboard::new().and_then(|mut cb| cb.set_text(text)) {
        Ok(_) => app.set_status(success_msg.to_string()),
        Err(_) => app.set_status("clipboard unavailable".to_string()),
    }
}

async fn handle_quickstart_key(app: &mut App, code: KeyCode) {
    let Modal::QuickStart(mut qs) = app.modal.clone() else {
        return;
    };

    if qs.step == 4 {
        match code {
            KeyCode::Char('c') => {
                qs.show_secret = !qs.show_secret;
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Char('i') => {
                if let Some(cid) = &qs.created_client_id.clone() {
                    copy_to_clipboard(app, cid, "client_id copied");
                }
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Char('s') => {
                if let Some(secret) = &qs.created_secret.clone() {
                    copy_to_clipboard(app, secret, "secret copied");
                }
                app.modal = Modal::QuickStart(qs);
            }
            KeyCode::Enter | KeyCode::Esc => {
                app.modal = Modal::None;
                load_tenants(app).await;
            }
            _ => {}
        }
        return;
    }

    let max_fields: usize = match qs.step {
        2 => 3,
        _ => 2,
    };

    match code {
        KeyCode::Esc => {
            app.modal = Modal::None;
        }
        KeyCode::Tab => {
            qs.field = (qs.field + 1) % max_fields;
            qs.error = None;
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Backspace => {
            pop_quickstart_field(&mut qs);
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Char(c) => {
            push_quickstart_field(&mut qs, c);
            app.modal = Modal::QuickStart(qs);
        }
        KeyCode::Enter => {
            qs.error = None;
            let client = app.client.clone();
            match qs.step {
                1 => {
                    if qs.tenant_name.is_empty() || qs.tenant_slug.is_empty() {
                        qs.error = Some("Name and Slug are required".to_string());
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    match client.create_tenant(&qs.tenant_name.clone(), &qs.tenant_slug.clone()).await {
                        Ok(t) => {
                            qs.created_tenant_id = Some(t.id);
                            qs.created_tenant_name = Some(qs.tenant_name.clone());
                            qs.step = 2;
                            qs.field = 0;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                2 => {
                    if qs.client_name.is_empty() || qs.redirect_uri.is_empty() {
                        qs.error = Some("Name and Redirect URI are required".to_string());
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    let Some(tid) = qs.created_tenant_id.clone() else { return };
                    let scopes: Vec<String> = qs.scopes.split_whitespace().map(|s| s.to_owned()).collect();
                    match client.create_client(&tid, &qs.client_name.clone(), vec![qs.redirect_uri.clone()], scopes).await {
                        Ok(c) => {
                            qs.created_client_id = Some(c.client_id);
                            qs.created_secret = c.client_secret;
                            qs.step = 3;
                            qs.field = 0;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                3 => {
                    if qs.user_email.is_empty() || qs.user_password.is_empty() {
                        qs.error = Some("Email and Password are required".to_string());
                        app.modal = Modal::QuickStart(qs);
                        return;
                    }
                    let Some(tid) = qs.created_tenant_id.clone() else { return };
                    match client.create_user(&tid, &qs.user_email.clone(), &qs.user_password.clone()).await {
                        Ok(_) => {
                            qs.step = 4;
                            app.modal = Modal::QuickStart(qs);
                        }
                        Err(e) => {
                            qs.error = Some(format!("{e}"));
                            app.modal = Modal::QuickStart(qs);
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn pop_quickstart_field(qs: &mut QuickStartState) {
    match (qs.step, qs.field) {
        (1, 0) => { qs.tenant_name.pop(); }
        (1, 1) => { qs.tenant_slug.pop(); }
        (2, 0) => { qs.client_name.pop(); }
        (2, 1) => { qs.redirect_uri.pop(); }
        (2, 2) => { qs.scopes.pop(); }
        (3, 0) => { qs.user_email.pop(); }
        (3, 1) => { qs.user_password.pop(); }
        _ => {}
    }
}

fn push_quickstart_field(qs: &mut QuickStartState, c: char) {
    match (qs.step, qs.field) {
        (1, 0) => qs.tenant_name.push(c),
        (1, 1) => qs.tenant_slug.push(c),
        (2, 0) => qs.client_name.push(c),
        (2, 1) => qs.redirect_uri.push(c),
        (2, 2) => qs.scopes.push(c),
        (3, 0) => qs.user_email.push(c),
        (3, 1) => qs.user_password.push(c),
        _ => {}
    }
}

async fn perform_edit_client(
    app: &mut App,
    id: String,
    name: String,
    redirect_uris_str: String,
    scopes_str: String,
) {
    let Some(tid) = app.active_tenant_id.clone() else { return };
    let redirect_uris: Vec<String> = redirect_uris_str
        .split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    let scopes: Vec<String> = scopes_str
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    match app.client.update_client(&tid, &id, &name, redirect_uris, scopes).await {
        Ok(_) => {
            app.set_status(format!("Client '{name}' updated"));
            load_clients(app, tid).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

async fn perform_edit_user(app: &mut App, id: String, is_active: bool) {
    let Some(tid) = app.active_tenant_id.clone() else { return };
    match app.client.set_user_active(&tid, &id, is_active).await {
        Ok(_) => {
            let status = if is_active { "activated" } else { "deactivated" };
            app.set_status(format!("User {status}"));
            load_users(app, tid).await;
        }
        Err(e) => app.modal = Modal::Error(format!("{e}")),
    }
}

async fn perform_delete(app: &mut App, id: String) {
    app.modal = Modal::None;
    let Some(tid) = app.active_tenant_id.clone() else {
        return;
    };
    match app.tab {
        Tab::Clients => {
            match app.client.deactivate_client(&tid, &id).await {
                Ok(_) => {
                    app.set_status("Client deactivated");
                    load_clients(app, tid).await;
                }
                Err(e) => app.modal = Modal::Error(format!("{e}")),
            }
        }
        Tab::Users => {
            match app.client.deactivate_user(&tid, &id).await {
                Ok(_) => {
                    app.set_status("User deactivated");
                    load_users(app, tid).await;
                }
                Err(e) => app.modal = Modal::Error(format!("{e}")),
            }
        }
        Tab::Health => {}
    }
}
