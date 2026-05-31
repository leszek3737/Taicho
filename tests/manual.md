# Manual E2E checklist

To be checked off by hand before tagging `v0.1.0` (MVP, M1–M4). Requires a running
Honcho server — local (`http://localhost:8000`) or cloud.

> Automated tests: `cargo test --all-features`. This file covers what we do not check
> automatically in the MVP (a real server + UI interactions).

## M1 — Profiles & connection
- [ ] Add a `local-dev` profile (base_url + workspace_id, no key).
- [ ] Add a `production` profile with an API key → the key lands in the Keychain, not in `profiles.json`.
- [ ] Connect → the status bar shows the workspace_id; Disconnect clears the state.
- [ ] Bad base_url (e.g. `localhost:8000` with no scheme) → a clear validation error.
- [ ] Restart the app → profiles preserved, key read from the Keychain.

## M2 — Peers
- [ ] Peer list with pagination (next/prev, page counter).
- [ ] Detail: Overview / Metadata / Config / Card / Representation (lazy) / Context / Sessions.
- [ ] Edit metadata (JSON) → save → refresh shows the change.
- [ ] Representation with options (search_query, top_k) returns a result.
- [ ] Context tab shows the openai / anthropic formats.

## M3 — Sessions & messages
- [ ] Session list + detail (Overview / Metadata / Config / Peers / Summaries / Context).
- [ ] Messages browser with pagination; copy content.
- [ ] Edit metadata of a single message (`update_message`) → visible after refresh.
- [ ] Summaries (short/long) and session context render.

## M4 — Workspaces & session peers
- [ ] Workspaces list; switch in the top bar.
- [ ] Deleting a workspace requires typing its name (double-confirm).
- [ ] Edit workspace metadata + config.
- [ ] Add/remove a peer from a session; set observe_me / observe_others.

## Edge states (every view)
- [ ] Loading: skeleton (not a per-record spinner).
- [ ] Empty: icon + sentence + CTA.
- [ ] Error: banner + `e.code()` + `Retry` when `is_retryable()`.
- [ ] Connection lost mid-operation → a clear error, no crash.
