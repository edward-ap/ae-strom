//! Cancellable HTTP transport. The decoder remains synchronous; HTTP/DNS run on
//! an owned async worker whose runtime and body task are joined on reader drop.
use std::{
    io::{self, Read},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::{Duration, Instant},
};
pub static ACTIVE_REQUESTS: AtomicUsize = AtomicUsize::new(0);
static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
mod buffer_timing;
pub use buffer_timing::HttpBufferStats;
pub(crate) use buffer_timing::HttpBufferTiming;

struct TransferTrace {
    id: u64,
    received: u64,
    reason: &'static str,
    report: Instant,
    buffer: Option<HttpBufferTiming>,
}
impl TransferTrace {
    fn end_reason(&mut self, reason: &'static str) {
        self.reason = reason;
    }
    fn progress(&mut self) {
        if self.report.elapsed() >= Duration::from_secs(5) {
            log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_progress","requestId":self.id,"receivedBytes":self.received,"buffer":self.buffer.as_ref().map(|b| b.snapshot())}));
            self.report = Instant::now();
        }
    }
}
impl Drop for TransferTrace {
    fn drop(&mut self) {
        if let Some(buffer) = &self.buffer {
            buffer.producer_ended();
        }
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_end","requestId":self.id,"receivedBytes":self.received,"reason":self.reason,"buffer":self.buffer.as_ref().map(|b| b.snapshot())}));
    }
}
type BodyMessage = io::Result<Vec<u8>>;
pub struct Request {
    url: String,
    headers: Vec<(String, String)>,
    cancel: Arc<AtomicBool>,
    collect_buffer_timing: bool,
}
pub struct Response {
    request_id: u64,
    headers: reqwest::header::HeaderMap,
    reader: BodyReader,
}
pub struct BodyReader {
    request_id: u64,
    delivered: u64,
    report: Instant,
    rx: mpsc::Receiver<BodyMessage>,
    pending: Vec<u8>,
    offset: usize,
    ended: bool,
    cancel: Arc<AtomicBool>,
    local: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    buffer_timing: Option<HttpBufferTiming>,
}
pub fn get(url: &str, cancel: Arc<AtomicBool>) -> Request {
    Request {
        url: url.into(),
        headers: Vec::new(),
        cancel,
        collect_buffer_timing: false,
    }
}
fn cancelled() -> io::Error {
    io::Error::new(io::ErrorKind::ConnectionAborted, "Request cancelled")
}
async fn cancellation(session: Arc<AtomicBool>, local: Arc<AtomicBool>) {
    while !session.load(Ordering::Acquire) && !local.load(Ordering::Acquire) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
impl Request {
    pub(crate) fn with_buffer_timing(mut self) -> Self {
        self.collect_buffer_timing = true;
        self
    }
    pub fn set(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
    pub fn call(self) -> io::Result<Response> {
        if self.cancel.load(Ordering::Acquire) {
            return Err(cancelled());
        }
        let (head_tx, head_rx) = mpsc::sync_channel(1);
        let (body_tx, body_rx) = mpsc::sync_channel(4);
        let local = Arc::new(AtomicBool::new(false));
        let worker_local = local.clone();
        let cancel = self.cancel.clone();
        let reader_cancel = cancel.clone();
        let request_id = NEXT_REQUEST.fetch_add(1, Ordering::Relaxed);
        let buffer_timing = self
            .collect_buffer_timing
            .then(|| HttpBufferTiming::new(request_id));
        let worker_buffer_timing = buffer_timing.clone();
        let caller = thread::current().name().map(str::to_owned);
        let worker = thread::Builder::new().name("radio-http".into()).spawn(move || {
            ACTIVE_REQUESTS.fetch_add(1, Ordering::AcqRel);
            struct Count;
            impl Drop for Count { fn drop(&mut self) { ACTIVE_REQUESTS.fetch_sub(1, Ordering::AcqRel); } }
            let _count = Count;
            let mut trace = TransferTrace { id: request_id, received: 0, reason: "cancelled", report: Instant::now(), buffer: worker_buffer_timing };
            log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_open","requestId":request_id,"caller":caller,"url":self.url,"requestHeaders":self.headers}));
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(r) => r,
                Err(e) => { trace.end_reason("runtime_error"); let _ = head_tx.send(Err(io::Error::other(e))); return; }
            };
            runtime.block_on(async move {
                let client = match reqwest::Client::builder().hickory_dns(true).connect_timeout(Duration::from_secs(4)).read_timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::limited(5)).build() {
                    Ok(c) => c,
                    Err(e) => { trace.end_reason("client_error"); let _ = head_tx.send(Err(io::Error::other(e.without_url()))); return; }
                };
                let mut request = client.get(&self.url);
                for (name, value) in self.headers { request = request.header(name, value); }
                let result = tokio::select! { _ = cancellation(cancel.clone(), worker_local.clone()) => Err(cancelled()), r = request.send() => r.map_err(|e| io::Error::other(e.without_url())) };
                let mut response = match result {
                    Ok(r) => r,
                    Err(e) => {
                        trace.end_reason("request_error");
                        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_error","requestId":request_id,"error":e.to_string(),"errorDetails":format!("{e:?}")}));
                        let _ = head_tx.send(Err(e)); return;
                    }
                };
                if log::log_enabled!(target: "ae_strom::http_trace", log::Level::Debug) {
                    let headers: Vec<_> = response.headers().iter().filter(|(name, _)| !matches!(name.as_str(), "set-cookie" | "authorization" | "proxy-authorization")).map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("<non-text>"))).collect();
                    log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_headers","requestId":request_id,"url":response.url().as_str(),"remote":response.remote_addr().map(|a| a.to_string()),"status":response.status().as_u16(),"headers":headers}));
                }
                if !response.status().is_success() {
                    trace.end_reason("http_status_error");
                    let _ = head_tx.send(Err(io::Error::new(if response.status().is_client_error() { io::ErrorKind::InvalidInput } else { io::ErrorKind::Other }, format!("HTTP status {}", response.status())))); return;
                }
                let mut ingress_adts = if trace.buffer.is_some()
                    && response.headers().get("content-type").and_then(|v| v.to_str().ok()).is_some_and(|v| v.to_ascii_lowercase().contains("aac"))
                    && crate::metadata::icy::interval(response.headers().get("icy-metaint").and_then(|v| v.to_str().ok())).is_ok()
                { Some(crate::audio::engine::AdtsClock::default()) } else { None };
                let mut ingress_icy = crate::metadata::icy::interval(response.headers().get("icy-metaint").and_then(|v| v.to_str().ok())).ok().flatten().map(crate::metadata::icy::AudioObserver::new);
                if let Some(buffer) = &trace.buffer { buffer.start(); }
                if head_tx.send(Ok(response.headers().clone())).is_err() { return; }
                loop {
                    let chunk = tokio::select! { _ = cancellation(cancel.clone(), worker_local.clone()) => break, r = response.chunk() => r };
                    let mut message = match chunk {
                        Ok(Some(bytes)) => {
                            trace.received += bytes.len() as u64;
                            if let Some(clock) = &mut ingress_adts {
                                if let Some(icy) = &mut ingress_icy { icy.feed(&bytes, |audio| clock.feed(audio)); }
                                else { clock.feed(&bytes); }
                            }
                            if let Some(buffer) = &trace.buffer { buffer.received(bytes.len(), ingress_adts.as_ref().map(|clock| &clock.stats)); }
                            Ok(bytes.to_vec())
                        }
                        Ok(None) => { trace.end_reason("eof"); break; }
                        Err(e) => {
                            trace.end_reason("body_error");
                            let error = e.without_url();
                            log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_error","requestId":request_id,"error":error.to_string(),"errorDetails":format!("{error:?}"),"isTimeout":error.is_timeout(),"isBody":error.is_body(),"isDecode":error.is_decode()}));
                            Err(io::Error::other(error))
                        }
                    };
                    let failed = message.is_err();
                    loop {
                        trace.progress();
                        if cancel.load(Ordering::Acquire) || worker_local.load(Ordering::Acquire) { return; }
                        let send_result = if let Some(buffer) = &trace.buffer { buffer.try_send(&body_tx, message) } else { body_tx.try_send(message) };
                        match send_result {
                            Ok(()) => break,
                            Err(mpsc::TrySendError::Disconnected(_)) => return,
                            Err(mpsc::TrySendError::Full(value)) => { message = value; tokio::time::sleep(Duration::from_millis(5)).await; }
                        }
                    }
                    if failed { break; }
                }
            });
        })?;
        let reader = BodyReader {
            request_id,
            delivered: 0,
            report: Instant::now(),
            rx: body_rx,
            pending: Vec::new(),
            offset: 0,
            ended: false,
            cancel: reader_cancel,
            local,
            worker: Some(worker),
            buffer_timing,
        };
        loop {
            if reader.cancel.load(Ordering::Acquire) {
                return Err(cancelled());
            }
            match head_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(headers)) => {
                    return Ok(Response {
                        request_id,
                        headers,
                        reader,
                    })
                }
                Ok(Err(e)) => return Err(e),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "HTTP worker ended before response",
                    ))
                }
            }
        }
    }
}
impl Response {
    pub(crate) fn buffer_timing(&self) -> Option<HttpBufferTiming> {
        self.reader.buffer_timing.clone()
    }
    pub(crate) fn request_id(&self) -> u64 {
        self.request_id
    }
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }
    pub fn into_reader(self) -> BodyReader {
        self.reader
    }
    pub fn into_string(self) -> io::Result<String> {
        let mut value = String::new();
        self.reader.take(1_048_577).read_to_string(&mut value)?;
        if value.len() > 1_048_576 {
            return Err(io::Error::other("Metadata response exceeds 1 MiB"));
        }
        Ok(value)
    }
}
impl Read for BodyReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        loop {
            if self.cancel.load(Ordering::Acquire) || self.local.load(Ordering::Acquire) {
                return Err(cancelled());
            }
            if self.offset < self.pending.len() {
                let n = out.len().min(self.pending.len() - self.offset);
                out[..n].copy_from_slice(&self.pending[self.offset..self.offset + n]);
                self.offset += n;
                self.delivered += n as u64;
                if let Some(buffer) = &self.buffer_timing {
                    buffer.delivered(n);
                }
                if self.report.elapsed() >= Duration::from_secs(5) {
                    log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_consumed","requestId":self.request_id,"deliveredBytes":self.delivered}));
                    self.report = Instant::now();
                }
                return Ok(n);
            }
            if self.ended {
                return Ok(0);
            }
            match self.rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(bytes)) => {
                    if let Some(buffer) = &self.buffer_timing {
                        buffer.take_from_channel(bytes.len());
                    }
                    self.pending = bytes;
                    self.offset = 0;
                }
                Ok(Err(e)) => {
                    self.ended = true;
                    return Err(e);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => self.ended = true,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    }
}
impl Drop for BodyReader {
    fn drop(&mut self) {
        log::debug!(target: "ae_strom::http_trace", "{}", serde_json::json!({"event":"http_reader_drop","requestId":self.request_id,"deliveredBytes":self.delivered,"bufferBeforeDrop":self.buffer_timing.as_ref().map(|b| b.snapshot())}));
        self.local.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if let Some(buffer) = &self.buffer_timing {
            buffer.reader_closed();
        }
    }
}
