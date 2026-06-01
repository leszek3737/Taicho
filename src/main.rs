mod actor;
mod app;
mod state;
mod telemetry;
mod ui;

fn main() -> anyhow::Result<()> {
    telemetry::init();
    taicho::persistence::secret_store::init_keyring();
    dioxus::launch(app::App);
    Ok(())
}
