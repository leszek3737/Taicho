use std::path::PathBuf;

use dioxus::prelude::*;
use taicho::domain::raw_json::RawJson;
use taicho::domain::upload::FileSource;
use taicho::domain::MessageRow;
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

/// Top-level upload panel.
///
/// Reads the active session + peer from `AppState::selection`, lets the user
/// pick files via [BrowseButton] / [DragDropZone], edit optional metadata via
/// [MetadataEditor], and dispatches one `Cmd::UploadFile` per chosen path.
async fn run_upload(
    state: AppState,
    actor: Coroutine<Cmd>,
    sid: String,
    pid: String,
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
    let mut failed: Option<AppError> = None;
    for (idx, path) in queued.into_iter().enumerate() {
        let path_key = path.clone();
        let (tx, rx) = oneshot::channel();
        actor.send(Cmd::UploadFile {
            session_id: sid.clone(),
            peer_id: pid.clone(),
            source: FileSource::Path(path),
            metadata: md_json.clone(),
            reply: tx,
        });
        let result: AppResult<MessageRow> = match rx.await {
            Ok(Ok(row)) => Ok(row),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::channel_closed("upload_file")),
        };
        match result {
            Ok(row) => {
                completed.push(row);
                files_sig.write().retain(|p| p != &path_key);
            }
            Err(e) => {
                failed = Some(e);
                break;
            }
        }
        progress_sig.set((idx + 1, total));
    }

    is_uploading_sig.set(false);
    match failed {
        Some(e) => {
            let msg = e.user_message();
            state.push_toast(ToastKind::Error, format!("Upload failed: {msg}"));
            last_error_sig.set(Some(msg));
            outcome_sig.set(UploadOutcome::Error);
        }
        None => {
            state.push_toast(ToastKind::Info, format!("Uploaded {} file(s)", completed.len()));
            files_sig.write().clear();
            outcome_sig.set(UploadOutcome::Done);
        }
    }
}

#[allow(clippy::too_many_lines)]
#[component]
pub fn UploadPanel() -> Element {
    let state: AppState = use_context();
    let actor: Coroutine<Cmd> = use_coroutine_handle::<Cmd>();

    let session_id = state.selection.session_id.read().clone();
    let peer_id = state.selection.peer_id.read().clone();

    let mut files: Signal<Vec<PathBuf>> = use_signal(Vec::new);
    let mut metadata: Signal<Option<RawJson>> = use_signal(|| None);
    let mut is_uploading: Signal<bool> = use_signal(|| false);
    let mut progress: Signal<(usize, usize)> = use_signal(|| (0, 0));
    let mut outcome: Signal<UploadOutcome> = use_signal(|| UploadOutcome::Idle);
    let mut last_error: Signal<Option<String>> = use_signal(|| None);

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
        let peer_id = peer_id.clone();
        let files = files;
        let metadata = metadata;
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
            let Some(pid) = peer_id.clone() else {
                last_error.set(Some("Select a peer first.".to_string()));
                outcome.set(UploadOutcome::Error);
                return;
            };
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
                pid,
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
            let msg = last_error.read().clone().unwrap_or_else(|| "Unknown error".to_string());
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

    if session_id.is_none() || peer_id.is_none() {
        return rsx! {
            EmptyView {
                title: "Nothing selected".to_string(),
                message: "Select a peer and a session before uploading.".to_string(),
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

    rsx! {
        section { class: "upload-panel",
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
                    class: "btn-primary",
                    onclick: trigger_upload,
                    disabled: false,
                    "Upload"
                }
                button {
                    class: "btn-secondary",
                    onclick: clear_files,
                    "Clear"
                }
            }
        }
    }
}
