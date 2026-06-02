use std::collections::HashSet;
use std::path::PathBuf;

use dioxus::prelude::*;
use taicho::domain::raw_json::RawJson;
use taicho::domain::upload::FileSource;
use taicho::domain::{MessageRow, SessionPeerRow};
use tokio::sync::oneshot;

use crate::actor::commands::Cmd;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, ToastKind};

use super::super::common::{EmptyView, ErrorView, LoadingView};

pub mod browse_button;
pub mod drag_drop_zone;
pub mod metadata_editor;

pub use browse_button::BrowseButton;
pub use drag_drop_zone::DragDropZone;
pub use metadata_editor::MetadataEditor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UploadOutcome {
    Idle,
    Done,
    Error,
}

struct PeerUploadResult {
    peer_id: String,
    success: bool,
    error: Option<String>,
}

/// Top-level upload panel.
///
/// Reads the active session from `AppState::selection`, fetches session peers
/// for multi-select, lets the user pick files via [BrowseButton] / [DragDropZone],
/// edit optional metadata via [MetadataEditor], and dispatches
/// `Cmd::UploadFileToMultiplePeers` per chosen path to all selected peers.
async fn run_upload(
    state: AppState,
    actor: Coroutine<Cmd>,
    sid: String,
    peer_ids: Vec<String>,
    queued: Vec<PathBuf>,
    md_json: Option<serde_json::Map<String, serde_json::Value>>,
    mut files_sig: Signal<Vec<PathBuf>>,
    mut is_uploading_sig: Signal<bool>,
    mut progress_sig: Signal<(usize, usize)>,
    mut outcome_sig: Signal<UploadOutcome>,
    mut last_error_sig: Signal<Option<String>>,
) {
    let total = queued.len();
    let mut completed: Vec<MessageRow> = Vec::new();
    let mut peer_results: Vec<PeerUploadResult> = Vec::new();
    let mut failed_file: Option<AppError> = None;

    for (idx, path) in queued.into_iter().enumerate() {
        let path_key = path.clone();
        let (tx, rx) = oneshot::channel();
        actor.send(Cmd::UploadFileToMultiplePeers {
            session_id: sid.clone(),
            peer_ids: peer_ids.clone(),
            source: FileSource::Path(path),
            metadata: md_json.clone(),
            reply: tx,
        });
        let result = match rx.await {
            Ok(r) => r,
            Err(_) => Err(AppError::channel_closed("upload_file_to_multiple_peers")),
        };
        match result {
            Ok(results) => {
                let mut file_ok = true;
                for (pid, peer_result) in results {
                    match peer_result {
                        Ok(row) => {
                            completed.push(row);
                            peer_results.push(PeerUploadResult {
                                peer_id: pid,
                                success: true,
                                error: None,
                            });
                        }
                        Err(e) => {
                            peer_results.push(PeerUploadResult {
                                peer_id: pid,
                                success: false,
                                error: Some(e.user_message()),
                            });
                            file_ok = false;
                        }
                    }
                }
                if file_ok {
                    files_sig.write().retain(|p| p != &path_key);
                }
            }
            Err(e) => {
                failed_file = Some(e);
                break;
            }
        }
        progress_sig.set((idx + 1, total));
    }

    is_uploading_sig.set(false);

    let has_failures = peer_results.iter().any(|r| !r.success) || failed_file.is_some();
    if let Some(e) = failed_file {
        let msg = e.user_message();
        state.push_toast(ToastKind::Error, format!("Upload failed: {msg}"));
        last_error_sig.set(Some(msg));
        outcome_sig.set(UploadOutcome::Error);
    } else if has_failures {
        let failed_peers: Vec<&str> = peer_results
            .iter()
            .filter(|r| !r.success)
            .map(|r| r.peer_id.as_str())
            .collect();
        let msg = format!(
            "Partial failure: {} of {} peer(s) failed ({})",
            failed_peers.len(),
            peer_ids.len(),
            failed_peers.join(", ")
        );
        state.push_toast(ToastKind::Error, msg.clone());
        last_error_sig.set(Some(msg));
        outcome_sig.set(UploadOutcome::Error);
    } else {
        state.push_toast(
            ToastKind::Info,
            format!(
                "Uploaded {} file(s) to {} peer(s)",
                completed.len() / peer_ids.len(),
                peer_ids.len()
            ),
        );
        files_sig.write().clear();
        outcome_sig.set(UploadOutcome::Done);
    }
}

#[allow(clippy::too_many_lines)]
#[component]
pub fn UploadPanel() -> Element {
    let state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    let session_id = state.selection.session_id.read().clone();

    let mut files: Signal<Vec<PathBuf>> = use_signal(Vec::new);
    let mut metadata: Signal<Option<RawJson>> = use_signal(|| None);
    let mut is_uploading: Signal<bool> = use_signal(|| false);
    let mut progress: Signal<(usize, usize)> = use_signal(|| (0, 0));
    let mut outcome: Signal<UploadOutcome> = use_signal(|| UploadOutcome::Idle);
    let mut last_error: Signal<Option<String>> = use_signal(|| None);
    let mut selected_peers: Signal<HashSet<String>> = use_signal(HashSet::new);

    let session_peers: Resource<Option<AppResult<Vec<SessionPeerRow>>>> =
        use_resource(move || {
            let sid = state.selection.session_id.read().clone();
            let actor = actor.clone();
            async move {
                let Some(sid) = sid else {
                    return None;
                };
                let (tx, rx) = oneshot::channel();
                actor.send(Cmd::ListSessionPeers {
                    session_id: sid,
                    reply: tx,
                });
                let result = match rx.await {
                    Ok(r) => r,
                    Err(_) => Err(AppError::channel_closed("list_session_peers")),
                };
                Some(result)
            }
        });

    let on_files_picked = move |picked: Vec<PathBuf>| {
        let mut current = files.write();
        current.extend(picked);
        outcome.set(UploadOutcome::Idle);
        last_error.set(None);
    };

    let on_metadata_change = move |next: Option<RawJson>| {
        metadata.set(next);
    };

    let clear_files = move |_| {
        files.write().clear();
        outcome.set(UploadOutcome::Idle);
        last_error.set(None);
        progress.set((0, 0));
    };

    let do_upload = {
        let actor = actor.clone();
        let session_id = session_id.clone();
        let files = files;
        let metadata = metadata;
        let selected_peers = selected_peers;
        let mut is_uploading = is_uploading;
        let mut progress = progress;
        let mut outcome = outcome;
        let mut last_error = last_error;
        let state = state.clone();
        move || {
            let Some(sid) = session_id.clone() else {
                last_error.set(Some("Select a session first.".to_string()));
                outcome.set(UploadOutcome::Error);
                return;
            };
            let peers: Vec<String> = selected_peers.read().iter().cloned().collect();
            if peers.is_empty() {
                last_error.set(Some("Select at least one peer.".to_string()));
                outcome.set(UploadOutcome::Error);
                return;
            }
            let queued: Vec<PathBuf> = files.read().clone();
            if queued.is_empty() {
                return;
            }

            let md_json = metadata
                .read()
                .as_ref()
                .and_then(RawJson::to_json_map);

            let total = queued.len();
            is_uploading.set(true);
            progress.set((0, total));
            outcome.set(UploadOutcome::Idle);
            last_error.set(None);

            spawn(run_upload(
                state.clone(),
                actor.clone(),
                sid,
                peers,
                queued,
                md_json,
                files,
                is_uploading,
                progress,
                outcome,
                last_error,
            ));
        }
    };

    let retry_upload = {
        let mut do_upload = do_upload.clone();
        move |_: MouseEvent| do_upload()
    };

    let trigger_upload = {
        let mut do_upload = do_upload.clone();
        move |_| do_upload()
    };

    if *is_uploading.read() {
        let (done, total) = *progress.read();
        return rsx! {
            LoadingView { label: format!("Uploading {done}/{total}...") }
        };
    }

    match *outcome.read() {
        UploadOutcome::Error => {
            let msg = last_error
                .read()
                .clone()
                .unwrap_or_else(|| "Unknown error".to_string());
            return rsx! {
                ErrorView {
                    code: "upload_failed".to_string(),
                    message: msg,
                    retryable: true,
                    on_retry: Some(EventHandler::new(retry_upload)),
                }
            };
        }
        _ => {}
    }

    if session_id.is_none() {
        return rsx! {
            EmptyView {
                title: "No session selected".to_string(),
                message: "Select a session before uploading.".to_string(),
            }
        };
    }

    let peer_list = match &*session_peers.read() {
        None => {
            return rsx! {
                LoadingView { label: "Loading session peers...".to_string() }
            };
        }
        Some(Err(e)) => {
            return rsx! {
                ErrorView {
                    code: e.code().to_string(),
                    message: e.user_message(),
                    retryable: e.is_retryable(),
                    on_retry: Some(EventHandler::new({
                        let session_peers = session_peers;
                        move |_: MouseEvent| session_peers.restart()
                    })),
                }
            };
        }
        Some(Ok(peers)) => peers.clone(),
    };

    if peer_list.is_empty() {
        return rsx! {
            EmptyView {
                title: "No peers in session".to_string(),
                message: "Add peers to the session before uploading files.".to_string(),
            }
        };
    }

    if files.read().is_empty() {
        let success_banner = if *outcome.read() == UploadOutcome::Done {
            Some(rsx! { div { class: "success-banner", "Upload complete." } })
        } else {
            None
        };
        return rsx! {
            {success_banner}
            PeerSelector {
                peers: peer_list,
                selected: selected_peers,
            }
            EmptyView {
                title: "No files queued".to_string(),
                message: "Drop files here or use the Browse button.".to_string(),
            }
            div { class: "upload-drop-area",
                DragDropZone { on_files: on_files_picked, accept: None }
                BrowseButton { on_picked: on_files_picked }
            }
        };
    }

    let selected_count = selected_peers.read().len();

    rsx! {
        section { class: "upload-panel",
            PeerSelector {
                peers: peer_list,
                selected: selected_peers,
            }

            div { class: "upload-drop-area",
                DragDropZone { on_files: on_files_picked, accept: None }
                BrowseButton { on_picked: on_files_picked }
            }

            if *outcome.read() == UploadOutcome::Done {
                div { class: "success-banner", "Upload complete." }
            }

            ul { class: "upload-file-list",
                for path in files.read().clone() {
                    li { key: "{path:?}", "{path:?}" }
                }
            }

            MetadataEditor { value: metadata, on_change: on_metadata_change }

            div { class: "upload-actions",
                button {
                    class: "primary-button",
                    onclick: trigger_upload,
                    disabled: selected_count == 0,
                    "Upload to {selected_count} peer(s)"
                }
                button {
                    class: "secondary-button",
                    onclick: clear_files,
                    "Clear"
                }
            }
        }
    }
}

#[component]
fn PeerSelector(
    peers: Vec<SessionPeerRow>,
    selected: Signal<HashSet<String>>,
) -> Element {
    rsx! {
        div { class: "peer-select",
            div { class: "peer-select-header",
                span { class: "peer-select-label", "Attach to peers:" }
                button {
                    class: "secondary-button",
                    onclick: move |_| {
                        let all_ids: HashSet<String> =
                            peers.iter().map(|p| p.id.clone()).collect();
                        if selected.read().len() == all_ids.len() {
                            selected.write().clear();
                        } else {
                            *selected.write() = all_ids;
                        }
                    },
                    if selected.read().len() == peers.len() {
                        "Deselect All"
                    } else {
                        "Select All"
                    }
                }
            }
            div { class: "peer-select-list",
                for peer in &peers {
                    {
                        let pid = peer.id.clone();
                        let is_checked = selected.read().contains(&pid);
                        rsx! {
                            label {
                                class: "peer-select-item",
                                key: "{pid}",
                                input {
                                    r#type: "checkbox",
                                    checked: is_checked,
                                    onchange: {
                                        let pid = pid.clone();
                                        move |_| {
                                            let mut sel = selected.write();
                                            if sel.contains(&pid) {
                                                sel.remove(&pid);
                                            } else {
                                                sel.insert(pid.clone());
                                            }
                                        }
                                    },
                                }
                                span { class: "peer-select-id", "{pid}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
