use futures_util::{Stream, StreamExt};
use honcho_ai::DialecticStream;
use tracing::debug;

use crate::actor::commands::StreamEvent;

pub async fn run_stream<S>(
    mut stream: DialecticStream<S>,
    tx: tokio::sync::mpsc::Sender<StreamEvent>,
) where
    S: Stream<Item = honcho_ai::error::Result<String>> + Unpin,
{
    let mut got_error = false;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(text) => {
                if tx.send(StreamEvent::Chunk(text)).await.is_err() {
                    debug!("stream chunk receiver dropped");
                    return;
                }
            }
            Err(e) => {
                let app_err: taicho::error::AppError = e.into();
                if tx.send(StreamEvent::Err(app_err)).await.is_err() {
                    debug!("stream error receiver dropped");
                    return;
                }
                got_error = true;
                break;
            }
        }
    }
    if !got_error {
        let final_text = stream.final_response().content().to_string();
        let _ = tx.send(StreamEvent::Done(final_text)).await;
    }
}
