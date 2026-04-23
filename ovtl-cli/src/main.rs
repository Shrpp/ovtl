mod api;
mod app;
mod components;
mod config_file;
mod events;
mod tui;
mod ui;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ovtl", about = "OVTL Admin TUI", version)]
struct Cli {
    /// OVTL server URL
    #[arg(long, short, env = "OVTL_URL")]
    url: Option<String>,

    /// Admin key (X-OVTL-Admin-Key)
    #[arg(long, short = 'k', env = "OVTL_ADMIN_KEY")]
    key: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let cfg = config_file::load();

    let url = cli
        .url
        .or(cfg.url)
        .unwrap_or_else(|| "http://localhost:3000".into());

    let admin_key = cli.key.or(cfg.admin_key).unwrap_or_default();

    if admin_key.is_empty() {
        eprintln!(
            "No admin key provided. Set OVTL_ADMIN_KEY, use --key, or add to ~/.config/ovtl/config.toml"
        );
        std::process::exit(1);
    }

    let client = api::Client::new(url, admin_key);
    let app_state = app::App::new(client);

    if let Err(e) = tui::run(app_state).await {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}
