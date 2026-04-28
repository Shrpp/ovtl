mod api;
mod app;
mod components;
mod config_file;
mod events;
mod tui;
mod ui;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ovlt", about = "OVLT Admin TUI", version)]
struct Cli {
    /// OVLT server URL
    #[arg(long, short, env = "OVLT_URL")]
    url: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg = config_file::load();

    let url = cli
        .url
        .or(cfg.url)
        .unwrap_or_else(|| "http://localhost:3000".into());

    let client = api::Client::new(url);
    let app_state = app::App::new(client);

    if let Err(e) = tui::run(app_state).await {
        eprintln!("TUI error: {e}");
        std::process::exit(1);
    }
}
