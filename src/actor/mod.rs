use dioxus::prelude::UnboundedReceiver;
use futures_util::StreamExt;
use honcho_ai::Honcho;
use tracing::{debug, warn};

use crate::actor::commands::Cmd;
use taicho::domain::WorkspaceInfo;
use taicho::error::AppResult;

pub mod client_factory;
pub mod commands;
pub mod dispatch;
pub mod stream;

#[tracing::instrument(skip(rx))]
pub async fn run_honcho_actor(mut rx: UnboundedReceiver<Cmd>) {
    tracing::info!("actor started");
    let mut client: Option<Honcho> = None;

    while let Some(cmd) = rx.next().await {
        match cmd {
            Cmd::Connect {
                profile,
                api_key,
                reply,
            } => {
                if client.is_some() {
                    let _ = reply.send(Err(taicho::error::AppError::Validation(
                        "already connected — disconnect first".to_owned(),
                    )));
                    continue;
                }
                let base_url = profile.base_url.clone();
                let result: AppResult<(Honcho, WorkspaceInfo)> = async {
                    let next = client_factory::build_client(&profile, api_key.as_deref())?;
                    next.force_ensure().await?;
                    let info = WorkspaceInfo {
                        id: next.workspace_id().to_owned(),
                        base_url,
                    };
                    Ok((next, info))
                }
                .await;

                match result {
                    Ok((next_client, info)) => {
                        if reply.send(Ok(info)).is_err() {
                            debug!("connect reply receiver dropped");
                        } else {
                            client = Some(next_client);
                        }
                    }
                    Err(e) => {
                        let _ = reply.send(Err(e));
                    }
                }
            }
            Cmd::Disconnect { reply } => {
                client = None;
                if reply.send(Ok(())).is_err() {
                    debug!("disconnect reply receiver dropped");
                }
            }
            other => {
                let Some(ref honcho) = client else {
                    dispatch::reply_not_connected(other);
                    continue;
                };
                dispatch::handle(honcho, other).await;
            }
        }
    }

    warn!("honcho actor stopped");
}
