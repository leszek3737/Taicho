use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueStatus {
    pub pending: u64,
    pub running: u64,
    pub completed: u64,
    pub sessions: u64,
    pub last_updated: Option<DateTime<Utc>>,
}

impl QueueStatus {
    /// Total work units (pending + running + completed). Does NOT include sessions.
    pub fn total(&self) -> u64 {
        self.pending + self.running + self.completed
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]
mod tests {
    use super::*;

    #[test]
    fn queue_status_default_is_empty() {
        let s = QueueStatus::default();
        assert_eq!(s.total(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn queue_status_total_sums_all_buckets() {
        let s = QueueStatus {
            pending: 2,
            running: 1,
            completed: 5,
            sessions: 3,
            last_updated: None,
        };
        assert_eq!(s.total(), 8);
        assert!(!s.is_empty());
    }
}
