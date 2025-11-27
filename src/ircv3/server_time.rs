use std::time::{SystemTime, UNIX_EPOCH};

pub fn format_server_time() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = now.as_secs();
    format_timestamp(secs)
}

pub fn format_timestamp(unix_secs: u64) -> String {
    if let Some(datetime) = chrono::DateTime::from_timestamp(unix_secs as i64, 0) {
        datetime.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    } else {
        "1970-01-01T00:00:00.000Z".to_string()
    }
}
