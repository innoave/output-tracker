use std::sync::atomic::{AtomicU64, Ordering};

static HANDLE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackerHandle(u64);

impl TrackerHandle {
    pub(crate) fn new() -> Self {
        Self(HANDLE_ID.fetch_add(1, Ordering::AcqRel))
    }
}
