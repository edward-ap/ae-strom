use crate::metadata::direct::{watch_direct_icy, DirectError};
use crate::metadata::model::{Info, StrategyHint};
use crate::metadata::sibling::{watch_sibling_icy, SiblingError};
use crate::metadata::status_json::{watch_status_json, JsonError};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
pub static ACTIVE_METADATA_WORKERS: AtomicUsize = AtomicUsize::new(0);
struct WorkerCount;
impl WorkerCount {
    fn start() -> Self {
        ACTIVE_METADATA_WORKERS.fetch_add(1, Ordering::AcqRel);
        Self
    }
}
impl Drop for WorkerCount {
    fn drop(&mut self) {
        ACTIVE_METADATA_WORKERS.fetch_sub(1, Ordering::AcqRel);
    }
}
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use url::Url;

/// Trait for logging metadata operations.
pub trait Logger: Send + Sync {
    fn log(&self, msg: &str);
}

/// Default logger implementation that uses the `log` crate.
pub struct LogLogger;
impl Logger for LogLogger {
    fn log(&self, msg: &str) {
        log::info!("[metadata] {msg}");
    }
}

/// Spawn a metadata worker thread that retries on failure.
///
/// The worker runs the provided closure in a loop, sleeping 2 seconds between
/// iterations or after errors. Stops when the stop_flag is set.
pub(crate) fn spawn_metadata_worker<F, E>(
    name: &'static str,
    stop_flag: Arc<AtomicBool>,
    mut f: F,
) -> thread::JoinHandle<()>
where
    F: FnMut() -> Result<(), E> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    thread::spawn(move || {
        let _count = WorkerCount::start();
        while !stop_flag.load(Ordering::SeqCst) {
            if let Err(e) = f() {
                log::warn!("[metadata:{name}] error: {e}");
            }
            for _ in 0..20 {
                if stop_flag.load(Ordering::Acquire) {
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    })
}

pub struct MetadataWatch {
    cancel: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl MetadataWatch {
    pub fn request_stop(&self) {
        self.cancel.store(true, Ordering::Release);
    }
    pub fn stop(&mut self) {
        self.request_stop();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
impl Drop for MetadataWatch {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Wrapper for metadata provider errors.
#[derive(Debug)]
pub enum MetadataError {
    Direct(DirectError),
    Json(JsonError),
    Sibling(SiblingError),
    Exhausted,
}

impl std::fmt::Display for MetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::Direct(e) => write!(f, "{e:?}"),
            MetadataError::Json(e) => write!(f, "{e:?}"),
            MetadataError::Sibling(e) => write!(f, "{e:?}"),
            MetadataError::Exhausted => write!(f, "exhausted"),
        }
    }
}

impl From<DirectError> for MetadataError {
    fn from(value: DirectError) -> Self {
        MetadataError::Direct(value)
    }
}

impl From<JsonError> for MetadataError {
    fn from(value: JsonError) -> Self {
        MetadataError::Json(value)
    }
}

impl From<SiblingError> for MetadataError {
    fn from(value: SiblingError) -> Self {
        MetadataError::Sibling(value)
    }
}

#[derive(Clone)]
/// Orchestrates metadata strategy selection and worker lifetime.
pub struct Provider {
    logger: Arc<dyn Logger>,
    json_poll_interval: Duration,
}

impl Provider {
    /// Create a new metadata provider with the specified JSON polling interval.
    ///
    /// The interval is clamped to a minimum of 5 seconds to avoid excessive polling.
    pub fn new(json_poll_interval: Duration) -> Self {
        Self {
            logger: Arc::new(LogLogger),
            json_poll_interval: json_poll_interval.max(Duration::from_secs(5)),
        }
    }

    /// Start watching metadata for a stream using hints, spawning worker threads.
    pub fn watch(
        &self,
        stream_url: String,
        hint: StrategyHint,
        on_info: impl Fn(Info) + Send + Sync + 'static,
        on_strategy_resolved: impl Fn(StrategyHint) + Send + Sync + 'static,
        stop_flag: Arc<AtomicBool>,
    ) -> MetadataWatch {
        let logger = self.logger.clone();
        let poll = self.json_poll_interval;
        let cancel = stop_flag.clone();
        let worker = thread::spawn(move || {
            let _count = WorkerCount::start();
            let on_info = Arc::new(on_info);
            let on_strategy_resolved = Arc::new(on_strategy_resolved);
            while !stop_flag.load(Ordering::Acquire) {
                let workers = Arc::new(Mutex::new(Vec::new()));
                let _ = Self::run(
                    logger.as_ref(),
                    stream_url.clone(),
                    hint.clone(),
                    on_info.clone(),
                    on_strategy_resolved.clone(),
                    poll,
                    stop_flag.clone(),
                    workers.clone(),
                );
                for worker in workers.lock().unwrap().drain(..) {
                    let _ = worker.join();
                }
                let started = std::time::Instant::now();
                while started.elapsed() < Duration::from_secs(2) {
                    if stop_flag.load(Ordering::Acquire) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(20));
                }
            }
        });
        MetadataWatch {
            cancel,
            worker: Some(worker),
        }
    }

    /// Attempt to derive a typical `status-json.xsl` URL for Icecast/Shoutcast.
    fn derive_status_json_url(stream_url: &str) -> Option<String> {
        if let Ok(parsed) = Url::parse(stream_url) {
            let path = parsed.path();
            // Keep everything up to and including the last '/'
            let base = if let Some(pos) = path.rfind('/') {
                &path[..=pos]
            } else {
                "/"
            };
            let mut new_url = parsed.clone();
            new_url.set_path(&format!("{base}status-json.xsl"));
            new_url.set_query(None);
            new_url.set_fragment(None);
            Some(new_url.to_string())
        } else {
            None
        }
    }

    fn run(
        logger: &dyn Logger,
        stream_url: String,
        hint: StrategyHint,
        on_info: Arc<dyn Fn(Info) + Send + Sync>,
        on_strategy_resolved: Arc<dyn Fn(StrategyHint) + Send + Sync>,
        json_poll_interval: Duration,
        stop_flag: Arc<AtomicBool>,
        workers: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
    ) -> Result<(), MetadataError> {
        // Helper to run a chosen strategy
        let try_icy = |url: &str| -> Result<(), MetadataError> {
            if stop_flag.load(Ordering::Acquire) {
                return Ok(());
            }
            watch_direct_icy(url, on_info.clone(), stop_flag.clone())
                .map(|worker| workers.lock().unwrap().push(worker))
                .map_err(Into::into)
        };

        let try_json = |url: &str| -> Result<(), MetadataError> {
            if stop_flag.load(Ordering::Acquire) {
                return Ok(());
            }
            watch_status_json(url, json_poll_interval, on_info.clone(), stop_flag.clone())
                .map(|worker| workers.lock().unwrap().push(worker))
                .map_err(Into::into)
        };

        // Use hint first (prefer cached metadata URL to avoid re-probing).
        if !hint.url.is_empty() {
            let hint_type = if hint.r#type.is_empty() {
                "ICY"
            } else {
                hint.r#type.as_str()
            };
            logger.log(&format!(
                "Using hinted metadata strategy: {} {}",
                hint_type, hint.url
            ));
            match hint_type {
                "ICY" => {
                    if try_icy(&hint.url).is_ok() {
                        on_strategy_resolved(StrategyHint {
                            r#type: hint_type.to_string(),
                            url: hint.url.clone(),
                        });
                        return Ok(());
                    }
                }
                "JSON" => {
                    if try_json(&hint.url).is_ok() {
                        on_strategy_resolved(StrategyHint {
                            r#type: hint_type.to_string(),
                            url: hint.url.clone(),
                        });
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        // Try direct ICY
        match try_icy(&stream_url) {
            Ok(()) => {
                on_strategy_resolved(StrategyHint {
                    r#type: "ICY".into(),
                    url: stream_url.clone(),
                });
                return Ok(());
            }
            Err(MetadataError::Direct(DirectError::NoIcyMetadata)) => {
                logger.log("No ICY metadata found on main stream");
            }
            Err(e) => {
                logger.log(&format!("Direct ICY failed: {e}"));
            }
        }

        // Try sibling ICY for flac
        if stream_url.to_lowercase().contains("flac") {
            match watch_sibling_icy(&stream_url, on_info.clone(), stop_flag.clone()) {
                Ok((hint, worker)) => {
                    workers.lock().unwrap().push(worker);
                    on_strategy_resolved(hint);
                    return Ok(());
                }
                Err(SiblingError::NoFlacInUrl) => {}
                Err(e) => logger.log(&format!("Sibling ICY failed: {e:?}")),
            }
        }

        // Try derived status-json.xsl on same host
        if let Some(status_url) = Self::derive_status_json_url(&stream_url) {
            logger.log(&format!("Trying derived JSON status at {}", status_url));
            match try_json(&status_url) {
                Ok(()) => {
                    on_strategy_resolved(StrategyHint {
                        r#type: "JSON".into(),
                        url: status_url,
                    });
                    return Ok(());
                }
                Err(_e) => {}
            }
        }

        // JSON fallback if hinted URL exists
        if !hint.url.is_empty() && hint.r#type == "JSON" && try_json(&hint.url).is_ok() {
            on_strategy_resolved(StrategyHint {
                r#type: "JSON".into(),
                url: hint.url.clone(),
            });
            return Ok(());
        }

        Err(MetadataError::Exhausted)
    }
}
