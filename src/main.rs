mod actor;
mod app;
mod state;
mod telemetry;
mod ui;

fn main() -> anyhow::Result<()> {
    telemetry::init();
    if let Err(e) = taicho::persistence::secret_store::init_keyring() {
        tracing::error!("failed to initialize keyring: {}", e.user_message());
        std::process::exit(1);
    }
    dioxus::launch(app::App);
    Ok(())
}
