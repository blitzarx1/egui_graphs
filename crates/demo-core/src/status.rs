use web_time::Instant;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct StatusMsg {
    pub kind: StatusKind,
    pub text: String,
    pub at: Instant,
    pub timeout_ms: u128,
}

impl StatusMsg {
    pub fn is_expired_at(&self, now: Instant) -> bool {
        self.at.elapsed().as_millis() > self.timeout_ms
            // Use provided now for deterministic tests
            || now.saturating_duration_since(self.at).as_millis() > self.timeout_ms
    }
}

#[derive(Default)]
pub struct StatusQueue {
    q: VecDeque<StatusMsg>,
}

impl StatusQueue {
    pub fn new() -> Self {
        Self { q: VecDeque::new() }
    }

    pub fn clear(&mut self) {
        self.q.clear();
    }

    pub fn push_custom(&mut self, kind: StatusKind, text: impl Into<String>, timeout_ms: u128) {
        self.q.push_back(StatusMsg {
            kind,
            text: text.into(),
            at: Instant::now(),
            timeout_ms,
        });
    }

    pub fn push_info(&mut self, text: impl Into<String>) {
        self.push_custom(StatusKind::Info, text, 3000);
    }
    pub fn push_success(&mut self, text: impl Into<String>) {
        self.push_custom(StatusKind::Success, text, 3500);
    }
    pub fn push_error(&mut self, text: impl Into<String>) {
        self.push_custom(StatusKind::Error, text, 4000);
    }

    pub fn retain_active(&mut self) {
        let now = Instant::now();
        self.retain_active_now(now);
    }

    pub fn retain_active_now(&mut self, now: Instant) {
        self.q.retain(|m| !m.is_expired_at(now));
    }

    /// Latest non-expired message (the most recent one).
    pub fn latest(&self) -> Option<&StatusMsg> {
        self.q.back()
    }

    pub fn len(&self) -> usize {
        self.q.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeouts_and_latest() {
        let mut sq = StatusQueue::new();
        sq.push_custom(StatusKind::Info, "i1", 100);
        sq.push_custom(StatusKind::Success, "ok1", 200);

        // Simulate time passing ~150ms
        let start = Instant::now();
        // Busy-wait a tiny amount to ensure monotonicity in wasm/instant
        while start.elapsed().as_millis() < 1 {}
        let now = Instant::now() + core::time::Duration::from_millis(150);
        sq.retain_active_now(now);

        // Info (100ms) should be gone, success should remain
        assert_eq!(sq.len(), 1);
        let latest = sq.latest().unwrap();
        assert_eq!(latest.kind, StatusKind::Success);
        assert_eq!(latest.text, "ok1");

        // Advance beyond 200ms
        let now2 = Instant::now() + core::time::Duration::from_millis(250);
        sq.retain_active_now(now2);
        assert_eq!(sq.len(), 0);
    }
}
