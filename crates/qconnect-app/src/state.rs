use qconnect_core::{PendingActionSlot, QConnectQueueState};

#[derive(Debug, Default)]
pub struct QconnectRuntimeState {
    pub queue: QConnectQueueState,
    pub pending: PendingActionSlot,
    pub transport_connected: bool,
    pub concurrency_canceled_action_uuid: Option<String>,
}
