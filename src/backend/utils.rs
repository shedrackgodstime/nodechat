use std::path::PathBuf;
use chrono::{TimeZone, Local, Datelike};

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

/// Formats a Unix timestamp (seconds) as "HH:MM".
/// Returns an empty string for timestamp 0.
pub fn format_hms(secs: i64) -> String {
    if secs == 0 { return String::new(); }
    let dt = Local.timestamp_opt(secs, 0).single().unwrap_or_else(|| Local.timestamp_opt(0,0).unwrap());
    dt.format("%H:%M").to_string()
}

/// Formats a Unix timestamp as a human-readable date label (e.g., "Today", "Yesterday", "October 24").
pub fn format_date_label(secs: i64) -> String {
    let dt = Local.timestamp_opt(secs, 0).single().unwrap_or_else(|| Local.timestamp_opt(0,0).unwrap());
    let now = Local::now();
    
    let dt_date = dt.date_naive();
    let now_date = now.date_naive();
    
    if dt_date == now_date {
        "Today".to_string()
    } else if dt_date == now_date.pred_opt().unwrap_or(now_date) {
        "Yesterday".to_string()
    } else if dt_date.year() == now_date.year() {
        dt.format("%B %e").to_string() 
    } else {
        dt.format("%B %e, %Y").to_string() 
    }
}

/// Derives a short human-readable name from a raw Peer ID or ticket.
pub fn derive_short_name(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.len() <= 12 {
        trimmed.to_string()
    } else {
        format!("Peer_{}", &trimmed[trimmed.len() - 6..])
    }
}

/// Secures a user PIN using SHA-256 hashing with domain separation.
pub fn secure_hash_pin(pin: &str) -> String {
    if pin.is_empty() {
        return String::new();
    }
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"nodechat-pin-v1:"); 
    hasher.update(pin.as_bytes());
    hex::encode(hasher.finalize())
}

/// Resolves the absolute path to the local SQLite database file.
#[cfg(not(target_os = "android"))]
pub fn resolve_database_path() -> PathBuf {
    use directories::ProjectDirs;
    let path = if let Some(proj) = ProjectDirs::from("com", "nodechat", "NodeChat") {
        let dir = proj.data_dir().to_path_buf();
        let _ = std::fs::create_dir_all(&dir);
        dir.join("nodechat.db")
    } else {
        PathBuf::from("nodechat.db")
    };
    path
}

/// Resolves the database path for Android systems.
#[cfg(target_os = "android")]
pub fn resolve_database_path() -> PathBuf {
    if let Some(data_dir) = crate::ANDROID_DATA_DIR.get() {
        data_dir.join("nodechat.db")
    } else {
        PathBuf::from("nodechat.db")
    }
}
