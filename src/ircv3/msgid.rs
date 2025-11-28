//! Message ID generation for IRCv3 message-ids capability.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static MSGID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique message ID string.
///
/// Returns a string like `1234567890-0` combining timestamp and counter.
pub fn generate_msgid() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let counter = MSGID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", timestamp, counter)
}
