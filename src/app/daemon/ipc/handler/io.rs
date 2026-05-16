use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::mpsc;

use super::dispatch::dispatch_request;
use crate::app::daemon::ipc::{
    DaemonRequest, DaemonResponse, DaemonResponseCode, PROTOCOL_VERSION,
};
use crate::app::daemon::supervisor::SupervisorEvent;

pub(super) async fn handle_connection(
    stream: &mut UnixStream,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    shutdown_tx: mpsc::Sender<()>,
) -> crate::app::Result<()> {
    let mut request_bytes = Vec::new();
    stream.read_to_end(&mut request_bytes).await?;
    let request = serde_json::from_slice::<DaemonRequest>(&request_bytes)?;
    if request.protocol_version != PROTOCOL_VERSION {
        let response = DaemonResponse::<serde_json::Value> {
            protocol_version: PROTOCOL_VERSION,
            ok: false,
            code: DaemonResponseCode::InvalidState,
            message: format!(
                "unsupported protocol version {} (expected {})",
                request.protocol_version, PROTOCOL_VERSION
            ),
            payload: None,
        };
        stream.write_all(&serde_json::to_vec(&response)?).await?;
        return Ok(());
    }

    let (encoded, should_shutdown) = dispatch_request(request.request, supervisor_tx).await?;
    stream.write_all(&encoded).await?;
    if should_shutdown {
        let _ = shutdown_tx.send(()).await;
    }
    Ok(())
}
