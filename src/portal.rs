use ashpd::desktop::screenshot;
use once_cell::sync::Lazy;
use tokio::sync::{mpsc, oneshot};

static PORTAL_TX: Lazy<mpsc::UnboundedSender<PortalRequest>> = Lazy::new(|| {
    let (tx, rx) = mpsc::unbounded_channel();
    std::thread::Builder::new()
        .name("portal-worker".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime for portal worker");
            rt.block_on(handle_portal_requests(rx));
        })
        .expect("Failed to spawn portal worker thread");
    tx
});

struct PortalRequest {
    window_id: String,
    responder: oneshot::Sender<Result<String, String>>,
}

pub async fn take_screenshot(window_id: String) -> Result<String, String> {
    let (tx, rx) = oneshot::channel();
    let _ = PORTAL_TX.send(PortalRequest {
        window_id,
        responder: tx,
    });
    rx.await.unwrap_or(Err("Portal channel closed".to_string()))
}

async fn handle_portal_requests(mut rx: mpsc::UnboundedReceiver<PortalRequest>) {
    while let Some(req) = rx.recv().await {
        let result = async {
            let id_type: ashpd::WindowIdentifierType = req
                .window_id
                .parse()
                .map_err(|e: ashpd::PortalError| e.to_string())?;
            let identifier = ashpd::WindowIdentifier::from(id_type);
            screenshot::ScreenshotRequest::default()
                .identifier(identifier)
                .interactive(true)
                .modal(false)
                .send()
                .await
                .and_then(|r| r.response())
                .map(|r| r.uri().to_string())
                .map_err(|e| e.to_string())
        }
        .await;

        let _ = req.responder.send(result);
    }
}
