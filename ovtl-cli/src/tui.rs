use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::{
    app::{App, Modal, Screen},
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

    let mut tenant_table = StatefulTable::new();
    let mut client_table = StatefulTable::new();

    // Initial data load
    load_tenants(&mut app).await;
    check_health(&mut app).await;

    loop {
        terminal.draw(|frame| {
            let (content_area, _header_area, statusbar_area) =
                ui::layout::split_areas(frame, &app);

            ui::layout::render_header(frame, &app, _header_area);
            ui::layout::render_nav(frame, &app, {
                let size = frame.area();
                use ratatui::layout::{Constraint, Direction, Layout};
                let outer = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Min(0),
                        ratatui::layout::Constraint::Length(1),
                    ])
                    .split(size);
                let body = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Length(16), Constraint::Min(0)])
                    .split(outer[1]);
                body[0]
            });

            let hints: Vec<(&str, &str)> = match app.screen {
                Screen::Tenants => ui::tenants::keybinds(),
                Screen::Clients => ui::clients::keybinds(),
                Screen::Health => ui::health::keybinds(),
            };

            match app.screen {
                Screen::Tenants => ui::tenants::render(frame, &app, content_area, &mut tenant_table),
                Screen::Clients => ui::clients::render(frame, &app, content_area, &mut client_table),
                Screen::Health => ui::health::render(frame, &app, content_area),
            }

            statusbar::render(frame, statusbar_area, &hints, app.status_msg.as_deref());

            // Overlay modals
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
                    modal::render_form(
                        frame,
                        "New Tenant",
                        &[("Name", name), ("Slug", slug)],
                        *field,
                    );
                }
                Modal::CreateClient { name, redirect_uri, scopes, field } => {
                    modal::render_form(
                        frame,
                        "New Client",
                        &[("Name", name), ("Redirect URI", redirect_uri), ("Scopes", scopes)],
                        *field,
                    );
                }
            }
        })?;

        match poll()? {
            Some(AppEvent::Key(key)) => {
                handle_key(&mut app, key.code, key.modifiers).await;
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

async fn handle_key(app: &mut App, code: KeyCode, _mods: KeyModifiers) {
    // Modal takes priority
    match app.modal.clone() {
        Modal::ConfirmDelete { id, label: _ } => {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    perform_delete(app, id).await;
                }
                _ => {
                    app.modal = Modal::None;
                }
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
                    if field == 0 { name.pop(); } else { slug.pop(); }
                    app.modal = Modal::CreateTenant { name, slug, field };
                }
                KeyCode::Char(c) => {
                    if field == 0 { name.push(c); } else { slug.push(c); }
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
                        0 => { name.pop(); }
                        1 => { redirect_uri.pop(); }
                        _ => { scopes.pop(); }
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
        Modal::None => {}
    }

    // Global keys
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('1') => {
            app.screen = Screen::Tenants;
            load_tenants(app).await;
        }
        KeyCode::Char('2') => {
            if let Some(t) = app.selected_tenant() {
                let tid = t.id.clone();
                app.screen = Screen::Clients;
                app.active_tenant_id = Some(tid.clone());
                load_clients(app, tid).await;
            } else {
                app.set_status("Select a tenant first (↑↓ then press →)");
            }
        }
        KeyCode::Char('3') => {
            app.screen = Screen::Health;
            check_health(app).await;
        }
        KeyCode::Right if app.screen == Screen::Tenants => {
            if let Some(t) = app.selected_tenant() {
                let tid = t.id.clone();
                app.screen = Screen::Clients;
                app.active_tenant_id = Some(tid.clone());
                load_clients(app, tid).await;
            }
        }
        KeyCode::Left if app.screen == Screen::Clients => {
            app.screen = Screen::Tenants;
        }
        KeyCode::Up => app.nav_up(),
        KeyCode::Down => app.nav_down(),
        KeyCode::Char('r') => match app.screen {
            Screen::Tenants => load_tenants(app).await,
            Screen::Clients => {
                if let Some(tid) = app.active_tenant_id.clone() {
                    load_clients(app, tid).await;
                }
            }
            Screen::Health => check_health(app).await,
        },
        KeyCode::Char('n') => match app.screen {
            Screen::Tenants => {
                app.modal = Modal::CreateTenant {
                    name: String::new(),
                    slug: String::new(),
                    field: 0,
                };
            }
            Screen::Clients => {
                if app.active_tenant_id.is_some() {
                    app.modal = Modal::CreateClient {
                        name: String::new(),
                        redirect_uri: String::new(),
                        scopes: String::from("openid email profile"),
                        field: 0,
                    };
                } else {
                    app.set_status("Select a tenant first");
                }
            }
            _ => {}
        },
        KeyCode::Char('d') if app.screen == Screen::Clients => {
            if let Some(c) = app.selected_client() {
                let id = c.id.clone();
                let label = c.name.clone();
                app.modal = Modal::ConfirmDelete { id, label };
            }
        }
        _ => {}
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
    app.clients_loading = true;
    match app.client.list_clients(&tenant_id).await {
        Ok(list) => {
            app.clients = list;
            app.client_selected = app.client_selected.min(app.clients.len().saturating_sub(1));
            app.clear_status();
        }
        Err(e) => app.set_status(format!("Error: {e}")),
    }
    app.clients_loading = false;
}

async fn check_health(app: &mut App) {
    match app.client.health().await {
        Ok(v) => {
            let status = v["status"].as_str().unwrap_or("ok").to_owned();
            app.health_status = Some(status);
        }
        Err(e) => {
            app.health_status = None;
            app.set_status(format!("Health check failed: {e}"));
        }
    }
}

async fn perform_create_tenant(app: &mut App, name: String, slug: String) {
    match app.client.create_tenant(&name, &slug).await {
        Ok(_) => {
            app.set_status(format!("Tenant '{name}' created"));
            load_tenants(app).await;
        }
        Err(e) => {
            app.modal = Modal::Error(format!("{e}"));
        }
    }
}

async fn perform_create_client(
    app: &mut App,
    name: String,
    redirect_uri: String,
    scopes_str: String,
) {
    let Some(tid) = app.active_tenant_id.clone() else { return };

    let scopes: Vec<String> = scopes_str.split_whitespace().map(|s| s.to_owned()).collect();
    match app.client.create_client(&tid, &name, vec![redirect_uri], scopes).await {
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
        Err(e) => {
            app.modal = Modal::Error(format!("{e}"));
        }
    }
}

async fn perform_delete(app: &mut App, id: String) {
    app.modal = Modal::None;
    let Some(tid) = app.active_tenant_id.clone() else { return };
    match app.client.deactivate_client(&tid, &id).await {
        Ok(_) => {
            app.set_status("Client deactivated");
            load_clients(app, tid).await;
        }
        Err(e) => {
            app.modal = Modal::Error(format!("{e}"));
        }
    }
}
