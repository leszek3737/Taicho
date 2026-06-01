# AGENT.md — Taicho

Rules for every AI agent (and human) working on this project. Read in full before your
first change. 

---

## 0. What this project is (and is NOT)

Taicho is a **native desktop client/inspector** for a Honcho server. All code is an SDK client.

**We do:** browse peers/sessions/messages/conclusions/representation, dialectic queries
(sync + stream), file upload, dreams, queue, view raw JSON metadata/config.

**We do NOT** (if you find yourself asking for this — decline and return to scope):
- we are not an end-user chat client,
- we do not host/configure the Honcho server or docker,
- we do not expose the server's LLM keys,
- we do not persist messages locally (beyond an in-memory cache),
- we do not create plain messages (`add_messages`) — ingest is via file upload only (M7),
- we do not edit message content — only a metadata patch is allowed (`update_message`),
- no webhooks and no `/v3/keys` — **SDK v0.1 does not support them**.

---

## 1. Build & Test

The order that matters before committing: **`fmt → clippy → test`**.

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features

# Dev with hot-reload (requires `cargo install dioxus-cli`):
dx serve --platform desktop --hotpatch

# Release bundle (macOS):
dx bundle --platform desktop --release --package-types "macos" --package-types "dmg"
```

The toolchain is pinned in `rust-toolchain.toml` (`stable` + `rustfmt`/`clippy`) — the
same for everyone and in CI. CI (`.github/workflows/ci-macos.yml`) runs on `macos-14`
with `RUSTFLAGS="-D warnings"`. Green CI is a merge requirement.

Quality rules are **machine-enforced** (Cargo `[lints]`, not just written here):
`unsafe_code = forbid`; `deny`d clippy lints `unwrap_used`, `panic`, `print_stdout`,
`print_stderr`, `dbg_macro`; plus `warn`s (`too_many_lines`, `needless_pass_by_value`,
`inefficient_to_string`, `module_inception`) that also fail CI under `-D warnings`. See §3.

---

## 2. Architecture — stick to it

Full description: BLUEPRINT.md §3–§7. A summary of the non-negotiable rules:

- **A single actor `use_coroutine(honcho_actor)`** in the root holds `Option<Honcho>` and
  is the only place that touches the SDK. Components talk to it **only** through `Cmd`
  (`use_coroutine_handle::<Cmd>()`); they never call the SDK directly.
- **Every `Cmd` carries `reply: oneshot::Sender<Result<T>>`** (or `tx: mpsc::Sender<_>` for
  a stream). Dropping the receiver = cancel; the actor detects it via `send().is_err()`
  and aborts.
- **All SDK → domain translation happens in `actor/dispatch.rs`** (and `stream.rs`). That
  is the only place that imports `honcho_ai::*` types, besides `client_factory.rs`.
- **The UI never consumes raw SDK types.** It works on `domain/*` (`PeerRow`,
  `PeerDetails`, `SessionRow`, `MessageRow`, …). This shields us from `#[non_exhaustive]`.
- **Data fetching in components: `use_resource`** (re-fires when dependent signals change).
  Refresh = `resource.restart()`. State: `read().is_none()` = loading.
- **Global state: `AppState` with `Signal<_>`**, injected via `use_context_provider`.

Directory layers: `actor/` (the only SDK touch point) · `domain/` (DTOs for the UI) ·
`state/` (signals) · `persistence/` (profiles + keyring) · `ui/` (components) · `error.rs`
· `telemetry.rs`. A new SDK feature = a new `Cmd` variant + a handler in `dispatch.rs` +
an optional type in `domain/` + a view in `ui/`. **Do not shortcut this path.**

---

## 3. Rust conventions

- **Edition 2024, latest stable Rust.** This is an application, not a published library,
  so it keeps **no MSRV contract** — always build on the newest stable (pinned to
  `stable` in `rust-toolchain.toml`; no `rust-version` in `Cargo.toml`). Use
  current-stable idioms freely; run `rustup update` to stay on the newest toolchain.
- **`unsafe` is FORBIDDEN** — `[lints.rust] unsafe_code = "forbid"`. A desktop client has
  no need for it. If you think you need it, that is a sign of bad API design.
- **No `unwrap`/`panic` in non-test code** — enforced by `[lints.clippy]`
  (`unwrap_used = "deny"`, `panic = "deny"`). Restructure: `let Some(x) = opt else
  { return Err(...) };` (let-else) or `opt.ok_or_else(|| AppError::...)?`.
- **`expect` is allowed ONLY at startup** (e.g. `init_keyring()` in `main.rs`) as a
  deliberate fail-fast — `expect_used` is intentionally not linted. Outside boot, use `?`.
  In `main()`, prefer returning `anyhow::Result` and using `?`.
- **No `println!`/`eprintln!`/`dbg!`** in app code — `print_stdout`, `print_stderr`,
  `dbg_macro` are `deny`. Use `tracing` (`info!`/`warn!`/`error!`) for all output.
- **`todo!()`/`unimplemented!()` are allowed** during scaffolding (M0–M4) — they are not
  linted yet. They will flip to `warn` after v0.1, so clear stubs before the MVP tag.
- **Tests may use `unwrap/expect/panic` and `eprintln!`** — add `#![allow(clippy::unwrap_used,
  clippy::expect_used, clippy::panic, clippy::print_stderr)]` at the top of the test file.
- **`#[allow(...)]` on a lint only for a genuine false-positive** — with a comment why.
  Do not use `#[allow]` to bypass deny-lints in non-test code.
- **Errors: a single `AppError` (thiserror)** with `#[from]` for `HonchoError`,
  `keyring_core`, `io`, `serde_json`. Keep `user_message()`, `is_retryable()`,
  `retry_after()`.
- **Imports ordered** (`reorder_imports = true`). Format with `cargo fmt`.
- **Naming:** types `CamelCase`, functions/fields `snake_case`, `Cmd` variants readable
  and grouped by a milestone comment (`// Peers (MVP)` etc.).
- **Code comments and identifiers in English.** (The whole repo is English-only.)
- **Pin critical versions deliberately:** `dioxus` and `honcho-ai` are breaking-change
  risks (BLUEPRINT §18). A bump = check the CHANGELOG, update `dispatch.rs` if needed.

---

## 4. SDK `honcho-ai` gotchas (verified — do not repeat these mistakes)

- **The `chat_stream(...)` builder has `.session(id)`, NOT `.session_id(id)`.** `.session_id()`
  exists only on `DialecticOptions::builder()` (the non-stream path). Mixing them up =
  a compile error / wrong routing.
- **`HonchoError` is `#[non_exhaustive]`.** Match on `e.code()` (a string), not on
  variants. Stable codes: `not_found`, `authentication_error`, `rate_limit_exceeded`,
  `validation_error`, `timeout`, `connection_error`, …
- **`Page::next_page()` returns `Result<Option<Page<T>>>`** — the `?` is mandatory. For
  "fetch everything" use `page.into_stream()`.
- **`DialecticStream::final_response()` returns `FinalResponse`** — text via `.content()`.
- **`Message` exposes accessors** (`.id()`, `.content()`, `.metadata()`, `.token_count()`…),
  not fields. Map to `MessageRow` in `dispatch.rs`.
- **`metadata` ≠ `configuration`.** The SDK separates them on workspace/peer/session.
  Tuning (reasoning/dream/summary) lives in *configuration*. Treat both as `RawJson` via
  `get/set_*_configuration_raw` to survive changes to the typed configs.
- **The context builder requires validation:** `SessionContextOptions::validate()?` after
  `.build()` when setting `peer_perspective`/`peer_target`.
- **`set_peer_configuration` requires the peer to already be in the session**
  (`add_peer`/`set_peers` first).
- **Client-side validations** (raised before the network): `page >= 1`, `size ∈ 1..=100`,
  query ≤ 10,000 chars, workspace_id `[a-zA-Z0-9_-]` 1..=512. Catch them as `validation_error`.
- **Do NOT use the `blocking` feature** — it panics inside a tokio runtime (and we are
  always inside Dioxus's async runtime). Always use the async `Honcho`.
- **SDK auto-retry** (2 attempts, backoff, respects `Retry-After`). Do not add your own
  retry loop for timeout/connection/429/5xx.

---

## 5. Security and persistence

- **API key only in the native keychain** via `keyring-core` + a per-platform store.
  **`init_keyring()` must run in `main.rs` BEFORE `dioxus::launch`** and before any
  `Entry::new` (BLUEPRINT §5). Without it the runtime panics.
- **A profile never holds the key in plain text.** The `uses_api_key: bool` field is just
  an indicator; the secret lives in the keychain under `dev.taicho` / `{profile_id}/api_key`.
- **Do not commit** `profiles.json`, `settings.json`, `window_state.json`, `.env` —
  they are in `.gitignore`. Do not log secrets or full base_urls containing a key.
- **URL validation in `profile_editor`** — guard against pasting `HONCHO_API_KEY` into `base_url`.
- **Destructive operations** (delete workspace/session) behind a `confirm_modal` with a
  double-confirm (type the name). Never delete without explicit user confirmation.

---

## 6. UI / Dioxus

- **Every list and detail has 3 explicit states:** Loading (skeleton, not a per-record
  spinner), Empty (icon + sentence + CTA), Error (banner + `e.code()` + `Retry` when
  `is_retryable()`).
- **Network operations never block the UI** — they go through the actor; the status bar
  shows progress.
- **Render streams incrementally**; navigating away = unmount = drop `rx` = cancel.
- **Styling: hand-written CSS + CSS variables** (decision from §14). No Tailwind / large
  class libraries.
- Keyboard shortcuts and the macOS menu per BLUEPRINT §11.

---

## 7. Tests

- **Unit (no network):** `profile_store` round-trip, `domain` mappers (SDK→UI), arg
  resolution in `client_factory` (builder > env > default).
- **Actor smoke:** a `HonchoLike` trait + a mock behind the `mock-honcho` feature; spawn
  the coroutine, `send(Cmd)`, expect a reply.
- **Integration:** behind the `TAICHO_HONCHO_URL` env var, `#[ignore]` by default. Skip
  gracefully when no server is available (`eprintln!("skipping integration test: ...")`).
- Test files may `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stderr)]`.
- **Every new `Cmd`** gets at least a domain-mapping test or a smoke test.

---

## 8. Git workflow

- **Do not commit/push without an explicit user request.** If you are on `main`, create a
  branch first.
- PRs per milestone (BLUEPRINT §16). MVP = M1–M4 → tag `v0.1.0`.
- Keep the diff within a single milestone/feature; do not mix refactoring with a feature.

---

## 9. Definition of Done (every change)

1. `cargo fmt --check` clean.
2. `cargo clippy --all-targets --all-features -- -D warnings` with no warnings.
3. `cargo test --all-features` green.
4. A new `Cmd` → a handler in `dispatch.rs` → (a `domain/` type) → a view/use in `ui/`.
5. No `unwrap/expect/panic` in non-test code; no secrets in logs/repo.
6. Loading/Empty/Error states handled in the touched view.
7. Compliance with the scope in §0 — nothing "extra" beyond the blueprint without approval.
<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **Taicho** (500 symbols, 1025 relationships, 41 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/Taicho/context` | Codebase overview, check index freshness |
| `gitnexus://repo/Taicho/clusters` | All functional areas |
| `gitnexus://repo/Taicho/processes` | All execution flows |
| `gitnexus://repo/Taicho/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
