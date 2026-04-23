use crate::api::{Client, OAuthClient, Tenant};

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Tenants,
    Clients,
    Health,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    None,
    CreateTenant { name: String, slug: String, field: usize },
    CreateClient {
        name: String,
        redirect_uri: String,
        scopes: String,
        field: usize,
    },
    ConfirmDelete { id: String, label: String },
    ShowSecret { client_id: String, secret: String },
    Error(String),
}

pub struct App {
    pub client: Client,
    pub screen: Screen,
    pub modal: Modal,

    // Tenants screen
    pub tenants: Vec<Tenant>,
    pub tenant_selected: usize,
    pub tenants_loading: bool,

    // Clients screen (scoped to selected tenant)
    pub clients: Vec<OAuthClient>,
    pub client_selected: usize,
    pub clients_loading: bool,
    pub active_tenant_id: Option<String>,

    // Health
    pub health_status: Option<String>,

    pub status_msg: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            screen: Screen::Tenants,
            modal: Modal::None,

            tenants: vec![],
            tenant_selected: 0,
            tenants_loading: false,

            clients: vec![],
            client_selected: 0,
            clients_loading: false,
            active_tenant_id: None,

            health_status: None,

            status_msg: None,
            should_quit: false,
        }
    }

    pub fn selected_tenant(&self) -> Option<&Tenant> {
        self.tenants.get(self.tenant_selected)
    }

    pub fn selected_client(&self) -> Option<&OAuthClient> {
        self.clients.get(self.client_selected)
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_msg = None;
    }

    pub fn nav_up(&mut self) {
        match self.screen {
            Screen::Tenants => {
                if self.tenant_selected > 0 {
                    self.tenant_selected -= 1;
                }
            }
            Screen::Clients => {
                if self.client_selected > 0 {
                    self.client_selected -= 1;
                }
            }
            _ => {}
        }
    }

    pub fn nav_down(&mut self) {
        match self.screen {
            Screen::Tenants => {
                if self.tenant_selected + 1 < self.tenants.len() {
                    self.tenant_selected += 1;
                }
            }
            Screen::Clients => {
                if self.client_selected + 1 < self.clients.len() {
                    self.client_selected += 1;
                }
            }
            _ => {}
        }
    }
}
