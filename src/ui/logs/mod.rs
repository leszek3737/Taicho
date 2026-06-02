use std::path::PathBuf;

use dioxus::prelude::*;

use super::common::{EmptyView, ErrorView, LoadingView};

const TAIL_LINES: usize = 200;
const REFRESH_INTERVAL_SECS: u64 = 5;

#[derive(Clone, Default)]
enum LogsState {
    #[default]
    Loading,
    Loaded(Vec<String>),
    Empty,
    Error(String, String),
}

fn log_dir_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "Taicho", "Taicho")?;
    Some(dirs.data_dir().join("logs"))
}

fn find_latest_log(log_dir: &std::path::Path) -> Option<PathBuf> {
    // Schwartzian transform: cache metadata before sorting to avoid O(N log N) disk I/O
    let mut entries: Vec<(PathBuf, std::time::SystemTime)> = std::fs::read_dir(log_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|name| name.starts_with("taicho.log"))
        })
        .filter_map(|path| {
            let modified = std::fs::metadata(&path).ok()?.modified().ok()?;
            Some((path, modified))
        })
        .collect();

    entries.sort_by_key(|b| std::cmp::Reverse(b.1));

    entries.into_iter().next().map(|(path, _)| path)
}

fn read_tail(path: &std::path::Path, limit: usize) -> Result<Vec<String>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    let skip = lines.len().saturating_sub(limit);
    Ok(lines.into_iter().skip(skip).collect())
}

async fn fetch_logs() -> LogsState {
    let Some(log_dir) = log_dir_path() else {
        return LogsState::Error(
            "no_data_dir".to_string(),
            "Could not determine application data directory.".to_string(),
        );
    };

    // Run all filesystem I/O inside spawn_blocking to avoid stalling the async executor
    match tokio::task::spawn_blocking(move || {
        if !log_dir.exists() {
            return Ok(None);
        }
        let latest = find_latest_log(&log_dir);
        match latest {
            None => Ok(None),
            Some(path) => read_tail(&path, TAIL_LINES).map(Some),
        }
    })
    .await
    {
        Ok(Ok(None)) => LogsState::Empty,
        Ok(Ok(Some(lines))) if lines.is_empty() => LogsState::Empty,
        Ok(Ok(Some(lines))) => LogsState::Loaded(lines),
        Ok(Err(e)) => LogsState::Error("io_error".to_string(), format!("Failed to read log: {e}")),
        Err(e) => LogsState::Error("task_error".to_string(), format!("Task join error: {e}")),
    }
}

#[component]
pub fn LogsPanel() -> Element {
    let mut logs_state = use_signal(|| LogsState::Loading);

    let refresh = use_callback(move |_: ()| {
        spawn(async move {
            logs_state.set(LogsState::Loading);
            let state = fetch_logs().await;
            logs_state.set(state);
        });
    });

    // Initial fetch + auto-refresh every 5 seconds
    use_future(move || {
        let refresh = refresh;
        async move {
            refresh.call(());
            let mut interval =
                tokio::time::interval(std::time::Duration::from_secs(REFRESH_INTERVAL_SECS));
            // Skip the immediate first tick to avoid double-refresh at startup
            interval.tick().await;
            loop {
                interval.tick().await;
                refresh.call(());
            }
        }
    });

    let snapshot = logs_state.read().clone();

    rsx! {
        div { class: "list-toolbar",
            h2 { "Logs" }
            button {
                class: "secondary-button",
                onclick: move |_| refresh.call(()),
                "Refresh"
            }
        }

        match snapshot {
            LogsState::Loading => rsx! {
                LoadingView { label: "Loading logs...".to_string() }
            },
            LogsState::Error(code, message) => rsx! {
                ErrorView {
                    code,
                    message,
                    retryable: false,
                    on_retry: None,
                }
            },
            LogsState::Empty => rsx! {
                EmptyView {
                    title: "No logs".to_string(),
                    message: "No log files found. Logs appear after the app writes to disk.".to_string(),
                }
            },
            LogsState::Loaded(lines) => rsx! {
                div { class: "logs-viewer",
                    div { class: "logs-info",
                        span { class: "muted", "{lines.len()} lines" }
                    }
                    pre { class: "logs-content",
                        for line in &lines {
                            {format!("{line}\n")}
                        }
                    }
                }
            },
        }
    }
}
