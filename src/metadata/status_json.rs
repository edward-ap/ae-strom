use crate::metadata::helpers::normalize_stream_title_case;
use crate::metadata::model::Info;
use crate::metadata::provider::spawn_metadata_worker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
/// Errors returned by polling JSON metadata endpoints.
pub enum JsonError {
    Http,
    Parse,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Http => write!(f, "http"),
            JsonError::Parse => write!(f, "parse"),
        }
    }
}

/// Watch a JSON status endpoint for metadata updates.
///
/// Fetches metadata immediately, then spawns a worker thread that polls
/// the endpoint at the specified interval. Common for Icecast/Shoutcast status pages.
///
/// # Errors
/// Returns `JsonError::Http` if the initial request fails or `JsonError::Parse`
/// if the response is not valid JSON.
pub fn watch_status_json(
    status_url: &str,
    poll_interval: Duration,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
    stop_flag: Arc<AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, JsonError> {
    if stop_flag.load(Ordering::Acquire) {
        return Ok(std::thread::spawn(|| {}));
    }
    let url = status_url.to_string();
    let poll = poll_interval.max(Duration::from_secs(5));

    if let Some(info) = fetch_once(&url, stop_flag.clone())? {
        on_info(info);
    }

    let on_info_clone = on_info.clone();
    let url_clone = url.clone();
    let stop_clone = stop_flag.clone();
    Ok(spawn_metadata_worker("json", stop_flag, move || {
        json_poll_loop(&url_clone, poll, on_info_clone.clone(), stop_clone.clone())
    }))
}

/// Fetch and parse metadata from a JSON endpoint once.
///
/// Returns None if the JSON contains no title field.
fn fetch_once(url: &str, stop_flag: Arc<AtomicBool>) -> Result<Option<Info>, JsonError> {
    let resp = crate::net::get(url, stop_flag)
        .call()
        .map_err(|_e| JsonError::Http)?;
    let body = resp.into_string().map_err(|_e| JsonError::Parse)?;
    let json_val: serde_json::Value = serde_json::from_str(&body).map_err(|_e| JsonError::Parse)?;

    if let Some(title) = extract_title(&json_val) {
        if !title.is_empty() {
            return Ok(Some(Info {
                title,
                station_name: None,
            }));
        }
    }
    Ok(None)
}

/// Polling loop that fetches JSON metadata at regular intervals.
///
/// Checks the stop flag before and after each fetch and sleep.
fn json_poll_loop(
    url: &str,
    poll: Duration,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), JsonError> {
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            return Ok(());
        }
        if let Ok(Some(info)) = fetch_once(url, stop_flag.clone()) {
            on_info(info);
        }
        if stop_flag.load(Ordering::SeqCst) {
            return Ok(());
        }
        let started = std::time::Instant::now();
        while started.elapsed() < poll {
            if stop_flag.load(Ordering::Acquire) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// Recursively extract a "title" field from JSON.
///
/// Handles both flat structures and nested Icecast/Shoutcast formats
/// (e.g., icestats.source as object or array).
fn extract_title(val: &serde_json::Value) -> Option<String> {
    if let Some(t) = val.get("title").and_then(|t| t.as_str()) {
        let s = t.trim();
        if !s.is_empty() {
            return Some(normalize_stream_title_case(s));
        }
    }

    if let Some(icestats) = val.get("icestats") {
        if let Some(title) = extract_title(icestats) {
            return Some(title);
        }
        if let Some(source) = icestats.get("source") {
            if let Some(arr) = source.as_array() {
                for src in arr {
                    if let Some(title) = extract_title(src) {
                        return Some(title);
                    }
                }
            } else if let Some(obj) = source.as_object() {
                if let Some(title) = extract_title(&serde_json::Value::Object(obj.clone())) {
                    return Some(title);
                }
            }
        }
    }

    None
}
