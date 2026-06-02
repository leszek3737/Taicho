# Taicho

A native desktop **admin/inspector for [Honcho](https://github.com/plastic-labs/honcho)** —
a "Postman for Honcho" for developers.

Taicho is **a client only**. It connects to an existing Honcho server instance
(local or cloud) via the [`honcho-ai`](https://crates.io/crates/honcho-ai) SDK. It does
not install, configure, or manage the server.

- **Stack:** Rust latest stable (edition 2024) · Dioxus 0.7 · `honcho-ai` 0.1 · Tokio
- **Target:** macOS first → Windows/Linux after v1.0
- **Plan:** see [BLUEPRINT.md](BLUEPRINT.md)
- **Rules for AI agents:** see [AGENT.md](AGENT.md)

## Requirements

- **Rust** latest stable (`rustup update stable`) — the app has no MSRV floor
- **dioxus-cli** matching 0.7: `cargo install dioxus-cli`
- **A running Honcho server** — local (`http://localhost:8000`) or cloud
  (`https://api.honcho.dev`). Taicho does not host it; you must provide an endpoint,
  a `workspace_id`, and optionally an API key.

## Run (dev)

```sh
dx serve --platform desktop --hotpatch
```

## Build (release, macOS)

```sh
dx bundle --platform desktop --release --package-types "macos" --package-types "dmg"
```

> **Gatekeeper:** builds before v0.2 are **unsigned**. macOS will show a warning.
> To run: right-click the `.app` → **Open** → confirm.

## First connection

1. Launch the app → *Connection* screen.
2. **New profile**: enter `name`, `base_url`, `workspace_id`, (optionally) an API key.
3. The API key is stored in the **macOS Keychain**, never in the plain-text profile file.
4. **Connect** → the status bar shows the workspace.

## Screenshots

<!-- TODO: Add screenshot — Main window showing the peer list sidebar and workspace overview -->
![Main window — peer list](assets/screenshots/main-window.png)

<!-- TODO: Add screenshot — Session detail view with message history and metadata -->
![Session detail with messages](assets/screenshots/session-detail.png)

<!-- TODO: Add screenshot — Chat/dialectic panel with streaming response -->
![Chat panel](assets/screenshots/chat-panel.png)

<!-- TODO: Add screenshot — Settings panel in dark theme -->
![Settings (dark)](assets/screenshots/settings-dark.png)

<!-- TODO: Add screenshot — Settings panel in light theme -->
![Settings (light)](assets/screenshots/settings-light.png)

<!-- TODO: Add screenshot — Command palette / search overlay -->
![Search palette](assets/screenshots/search-palette.png)

<!-- TODO: Add screenshot — Conclusions view for a session -->
![Conclusions view](assets/screenshots/conclusions.png)

> To capture: run the app with `dx serve`, open each view, and save a PNG to
> `assets/screenshots/`. Retina (2x) preferred.

## License

MIT — see [LICENSE](LICENSE). Not affiliated with Plastic Labs.
