//! Cancellable HTTP transport. The decoder remains synchronous; HTTP/DNS run on
//! an owned async worker whose runtime and body task are joined on reader drop.
use std::{
    io::{self, Read},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};
pub static ACTIVE_REQUESTS: AtomicUsize = AtomicUsize::new(0);
type BodyMessage = io::Result<Vec<u8>>;
pub struct Request {
    url: String,
    headers: Vec<(String, String)>,
    cancel: Arc<AtomicBool>,
}
pub struct Response {
    headers: reqwest::header::HeaderMap,
    reader: BodyReader,
}
pub struct BodyReader {
    rx: mpsc::Receiver<BodyMessage>,
    pending: Vec<u8>,
    offset: usize,
    ended: bool,
    cancel: Arc<AtomicBool>,
    local: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
pub fn get(url: &str, cancel: Arc<AtomicBool>) -> Request {
    Request {
        url: url.into(),
        headers: Vec::new(),
        cancel,
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
        let worker=thread::Builder::new().name("radio-http".into()).spawn(move||{
            ACTIVE_REQUESTS.fetch_add(1,Ordering::AcqRel);
            struct Count;impl Drop for Count{fn drop(&mut self){ACTIVE_REQUESTS.fetch_sub(1,Ordering::AcqRel);}}
            let _count=Count;
            let runtime=match tokio::runtime::Builder::new_current_thread().enable_all().build(){Ok(r)=>r,Err(e)=>{let _=head_tx.send(Err(io::Error::other(e)));return;}};
            runtime.block_on(async move{
                let client=match reqwest::Client::builder().hickory_dns(true).connect_timeout(Duration::from_secs(4)).read_timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::limited(5)).build(){Ok(c)=>c,Err(e)=>{let _=head_tx.send(Err(io::Error::other(e.without_url())));return;}};
                let mut request=client.get(&self.url);
                for(name,value)in self.headers{request=request.header(name,value);}
                let result=tokio::select!{_ = cancellation(cancel.clone(),worker_local.clone())=>Err(cancelled()),r=request.send()=>r.map_err(|e|io::Error::other(e.without_url()))};
                let mut response=match result{Ok(r)if r.status().is_success()=>r,Ok(r)=>{let _=head_tx.send(Err(io::Error::new(if r.status().is_client_error(){io::ErrorKind::InvalidInput}else{io::ErrorKind::Other},format!("HTTP status {}",r.status()))));return;},Err(e)=>{let _=head_tx.send(Err(e));return;}};
                if head_tx.send(Ok(response.headers().clone())).is_err(){return;}
                loop {
                    let chunk=tokio::select!{_ = cancellation(cancel.clone(),worker_local.clone())=>break,r=response.chunk()=>r};
                    let mut message=match chunk{Ok(Some(bytes))=>Ok(bytes.to_vec()),Ok(None)=>break,Err(e)=>Err(io::Error::other(e.without_url()))};
                    let failed=message.is_err();
                    loop {
                        if cancel.load(Ordering::Acquire)||worker_local.load(Ordering::Acquire){return;}
                        match body_tx.try_send(message){Ok(())=>break,Err(mpsc::TrySendError::Disconnected(_))=>return,Err(mpsc::TrySendError::Full(value))=>{message=value;tokio::time::sleep(Duration::from_millis(5)).await;}}
                    }
                    if failed{break;}
                }
            });
            // Runtime drop cancels DNS/connection tasks too; there is no detached
            // std::net::ToSocketAddrs resolver on the cancellation path.
        })?;
        let reader = BodyReader {
            rx: body_rx,
            pending: Vec::new(),
            offset: 0,
            ended: false,
            cancel: reader_cancel,
            local,
            worker: Some(worker),
        };
        loop {
            if reader.cancel.load(Ordering::Acquire) {
                return Err(cancelled());
            }
            match head_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(headers)) => return Ok(Response { headers, reader }),
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
                return Ok(n);
            }
            if self.ended {
                return Ok(0);
            }
            match self.rx.recv_timeout(Duration::from_millis(20)) {
                Ok(Ok(bytes)) => {
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
        self.local.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
