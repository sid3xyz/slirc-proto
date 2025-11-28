//! Batch reference generation for IRCv3 BATCH command.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static BATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique batch reference string.
///
/// Returns a string like `1234567890-0` combining timestamp and counter.
pub fn generate_batch_ref() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let counter = BATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", timestamp, counter)
}
