use async_trait::async_trait;
use qconnect_core::{QConnectQueueState, QConnectRendererState, RendererCommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QconnectAppEvent {
    TransportConnected,
    TransportDisconnected,
    QueueUpdated(QConnectQueueState),
    RendererUpdated(QConnectRendererState),
    RendererCommandApplied {
        command: RendererCommand,
        state: QConnectRendererState,
    },
    PendingActionStarted {
        uuid: String,
    },
    PendingActionCompleted {
        uuid: String,
    },
    PendingActionTimedOut {
        uuid: String,
        timeout_ms: u64,
    },
    PendingActionCanceledByConcurrentRemoteEvent {
        pending_uuid: String,
        remote_action_uuid: String,
    },
    QueueErrorIgnoredByConcurrency {
        action_uuid: String,
    },
    QueueResyncTriggered,
}

#[async_trait]
pub trait QconnectEventSink: Send + Sync {
    async fn on_event(&self, event: QconnectAppEvent);
}

#[derive(Debug, Clone, Default)]
pub struct NoOpEventSink;

#[async_trait]
impl QconnectEventSink for NoOpEventSink {
    async fn on_event(&self, _event: QconnectAppEvent) {}
}
