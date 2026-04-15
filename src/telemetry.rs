use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct Telemetry {
    blocked_network_attempts: AtomicU64,
}

impl Telemetry {
    pub fn increment_blocked_network_attempts(&self) {
        self.blocked_network_attempts
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn blocked_network_attempts(&self) -> u64 {
        self.blocked_network_attempts.load(Ordering::Relaxed)
    }
}
