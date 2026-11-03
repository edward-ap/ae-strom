//! Network fixtures bind loopback only. TestSink decodes real PCM without
//! enumerating/opening CPAL devices. No public radio URL appears in these tests.
use crate::{
    controller::{Action, Controller, PlaybackMode},
    metadata::{Provider, StrategyHint},
    net, session,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
static SERIAL: Mutex<()> = Mutex::new(());
struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    requests: Arc<AtomicUsize>,
}
fn response(stream: &mut TcpStream, headers: &str, body: &[u8]) {
    let _ = stream
        .write_all(format!("HTTP/1.1 200 OK\r\nConnection: close\r\n{headers}\r\n").as_bytes());
    let _ = stream.write_all(body);
}
fn wave_header(frames: u32) -> Vec<u8> {
    let size = frames * 4;
    let mut out = Vec::new();
    out.extend(b"RIFF");
    out.extend((size + 36).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(48_000u32.to_le_bytes());
    out.extend(192_000u32.to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend(size.to_le_bytes());
    out
}
impl Server {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let requests = Arc::new(AtomicUsize::new(0));
        let counts = requests.clone();
        let worker = thread::spawn(move || {
            let mut clients = Vec::new();
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let stop = stopped.clone();
                        let count = counts.clone();
                        clients.push(thread::spawn(move||{
            stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();stream.set_write_timeout(Some(Duration::from_millis(100))).unwrap();
            let mut head=Vec::new();let mut one=[0u8;1];while head.len()<8192&&!head.ends_with(b"\r\n\r\n")&&!stop.load(Ordering::Acquire){match stream.read(&mut one){Ok(0)=>return,Ok(_)=>head.push(one[0]),Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut)=>{},Err(_)=>return}}
            let request=String::from_utf8_lossy(&head);let path=request.split_whitespace().nth(1).unwrap_or("/");let number=count.fetch_add(1,Ordering::AcqRel);
            match path {
                "/headers"=>while !stop.load(Ordering::Acquire){thread::sleep(Duration::from_millis(10));},
                "/trickle"=>{let _=stream.write_all(b"HTTP/1.1 200 OK\r\nX-Slow: ");while !stop.load(Ordering::Acquire){if stream.write_all(b"a").is_err(){break;}thread::sleep(Duration::from_millis(20));}},
                "/body"=>{response(&mut stream,"Content-Type: audio/wav\r\nContent-Length: 4000044\r\n",&[]);while !stop.load(Ordering::Acquire){thread::sleep(Duration::from_millis(10));}},
                "/pcm"|"/eof"=>{
                    let frames=if path=="/eof"{8192}else{1_000_000};let header=wave_header(frames);response(&mut stream,&format!("Content-Type: audio/wav\r\nContent-Length: {}\r\n",frames*4+44),&header);
                    let mut block=Vec::with_capacity(2048);for i in 0..512{let sample=(((i%64) as f32/64.-0.5)*16000.) as i16;block.extend(sample.to_le_bytes());block.extend(sample.to_le_bytes());}
                    let mut written=0;while written<frames*4&&!stop.load(Ordering::Acquire){let n=(frames as usize*4-written as usize).min(block.len());if stream.write_all(&block[..n]).is_err(){break;}written+=n as u32;thread::sleep(Duration::from_millis(4));}
                },
                "/late" if number < 3 => {let _=stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");},
                "/icy"|"/late"=>{
                    response(&mut stream,"icy-metaint: 16\r\nicy-name: Fixture Radio\r\n",&[]);
                    let mut block=vec![0u8;16];block.push(2);let mut meta=b"StreamTitle='Second';".to_vec();meta.resize(32,0);
                    if number==0{block.extend(&meta[..8]);let _=stream.write_all(&block);}else{block.extend(meta);while !stop.load(Ordering::Acquire){if stream.write_all(&block).is_err(){break;}thread::sleep(Duration::from_millis(50));}}
                },
                "/json"=>{let body=b"{\"title\":\"Fixture Artist - JSON Track\"}";response(&mut stream,&format!("Content-Type: application/json\r\nContent-Length: {}\r\n",body.len()),body);},
                _=>{let _=stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");},
            }
        }));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
            for client in clients {
                client.join().unwrap();
            }
        });
        Self {
            url,
            stop,
            worker: Some(worker),
            requests,
        }
    }
    fn path(&self, path: &str) -> String {
        format!("{}{}", self.url, path)
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
fn wait(mut predicate: impl FnMut() -> bool, limit: Duration) {
    let started = Instant::now();
    while !predicate() {
        assert!(
            started.elapsed() < limit,
            "condition timed out after {limit:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
fn directory() -> PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("backend-network-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn actor() -> (Controller, PathBuf) {
    let dir = directory();
    let controller = Controller::start(
        dir.join("config.json"),
        PlaybackMode::TestSink,
        Arc::new(|_| {}),
    )
    .unwrap();
    (controller, dir)
}
fn action(controller: &Controller, action: Action) -> crate::controller::Snapshot {
    let started = Instant::now();
    let result = controller
        .submit(action)
        .unwrap()
        .recv_timeout(Duration::from_millis(500))
        .unwrap()
        .unwrap();
    assert!(started.elapsed() < Duration::from_millis(500));
    result
}
fn clean(controller: Controller, dir: PathBuf) {
    let started = Instant::now();
    controller.shutdown().unwrap();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(
        session::worker_counts(),
        serde_json::json!({"http":0,"metadata":0,"audio":0,"sessionOwners":0})
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn transport_cancels_stalled_headers_trickle_and_stalled_body_without_orphans() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    for path in ["/headers", "/trickle", "/body"] {
        let cancel = Arc::new(AtomicBool::new(false));
        let stopping = cancel.clone();
        let url = server.path(path);
        let before = server.requests.load(Ordering::Acquire);
        let worker = thread::spawn(move || {
            if let Ok(response) = net::get(&url, stopping).call() {
                let mut reader = response.into_reader();
                let _ = reader.read(&mut [0u8; 1]);
            }
        });
        wait(
            || server.requests.load(Ordering::Acquire) > before,
            Duration::from_secs(2),
        );
        let started = Instant::now();
        cancel.store(true, Ordering::Release);
        worker.join().unwrap();
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "{path} cancellation was slow"
        );
        assert_eq!(net::ACTIVE_REQUESTS.load(Ordering::Acquire), 0);
    }
}

#[test]
fn actor_remains_responsive_during_probe_switches_and_shutdown() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/headers"),
        },
    );
    action(
        &controller,
        Action::SetStationUrl {
            bank: 1,
            slot: 0,
            url: server.path("/trickle"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || server.requests.load(Ordering::Acquire) > 0,
        Duration::from_secs(2),
    );
    for _ in 0..6 {
        action(&controller, Action::SelectStation { bank: 1, slot: 0 });
        action(&controller, Action::SetVolume { volume: 17 });
        action(&controller, Action::SetMuted { muted: true });
        action(&controller, Action::SelectStation { bank: 0, slot: 0 });
    }
    let stopped = action(&controller, Action::SetPlaying { playing: false });
    assert!(!stopped.requested_playing);
    action(&controller, Action::SetPlaying { playing: true });
    action(&controller, Action::SetMuted { muted: false });
    clean(controller, dir);
}

#[test]
fn actual_pcm_decode_gain_mute_eq_and_eof_reconnect_use_no_device() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/pcm"),
        },
    );
    action(&controller, Action::SetVolume { volume: 10 });
    action(
        &controller,
        Action::SetEq {
            preamp: -6.,
            bands: [0.; 10],
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || {
            controller.diagnostics().unwrap()["engine"]["outputSamples"]
                .as_u64()
                .unwrap()
                > 5000
        },
        Duration::from_secs(4),
    );
    let diag = controller.diagnostics().unwrap();
    assert_eq!(diag["engine"]["headless"], true);
    assert!(diag["engine"]["decodedSamples"].as_u64().unwrap() > 0);
    assert!(diag["engine"]["peak"].as_f64().unwrap() > 0.);
    assert!(diag["engine"]["peak"].as_f64().unwrap() < 0.007);
    action(&controller, Action::SetMuted { muted: true });
    thread::sleep(Duration::from_millis(40));
    let muted = controller.diagnostics().unwrap();
    thread::sleep(Duration::from_millis(100));
    let later = controller.diagnostics().unwrap();
    assert!(
        later["engine"]["outputSamples"].as_u64().unwrap()
            > muted["engine"]["outputSamples"].as_u64().unwrap()
    );
    assert_eq!(
        later["engine"]["nonSilentSamples"],
        muted["engine"]["nonSilentSamples"]
    );
    action(&controller, Action::SetMuted { muted: false });
    action(&controller, Action::SetPlaying { playing: false });
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/eof"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || {
            controller.diagnostics().unwrap()["engine"]["connections"]
                .as_u64()
                .unwrap()
                >= 2
        },
        Duration::from_secs(5),
    );
    clean(controller, dir);
}

#[test]
fn icy_reopens_after_partial_frame_and_json_workers_join() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let titles = Arc::new(Mutex::new(Vec::new()));
    let received = titles.clone();
    let cancel = Arc::new(AtomicBool::new(false));
    let mut watch = Provider::new(Duration::from_secs(5)).watch(
        server.path("/icy"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        move |info| received.lock().unwrap().push(info.title),
        |_| {},
        cancel,
    );
    wait(
        || titles.lock().unwrap().iter().any(|title| title == "Second"),
        Duration::from_secs(6),
    );
    let started = Instant::now();
    watch.stop();
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(net::ACTIVE_REQUESTS.load(Ordering::Acquire), 0);
    let count = Arc::new(AtomicUsize::new(0));
    let received = count.clone();
    let mut watch = Provider::new(Duration::from_secs(5)).watch(
        server.path("/json"),
        StrategyHint {
            r#type: "JSON".into(),
            url: server.path("/json"),
        },
        move |_| {
            received.fetch_add(1, Ordering::AcqRel);
        },
        |_| {},
        Arc::new(AtomicBool::new(false)),
    );
    wait(|| count.load(Ordering::Acquire) > 0, Duration::from_secs(2));
    let started = Instant::now();
    watch.stop();
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
    assert_eq!(net::ACTIVE_REQUESTS.load(Ordering::Acquire), 0);
}

#[test]
fn permanent_http_error_is_terminal_and_user_can_retry() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/missing"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().status == "error",
        Duration::from_secs(3),
    );
    assert!(!controller.snapshot().requested_playing);
    assert!(controller.snapshot().error.is_some());
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/pcm"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().status == "playing",
        Duration::from_secs(4),
    );
    clean(controller, dir);
}

#[test]
fn metadata_recovers_when_initial_discovery_is_unavailable() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let titles = Arc::new(Mutex::new(Vec::new()));
    let received = titles.clone();
    let mut watch = Provider::new(Duration::from_secs(5)).watch(
        server.path("/late"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/late"),
        },
        move |info| received.lock().unwrap().push(info.title),
        |_| {},
        Arc::new(AtomicBool::new(false)),
    );
    wait(
        || titles.lock().unwrap().iter().any(|title| title == "Second"),
        Duration::from_secs(6),
    );
    watch.stop();
    assert!(server.requests.load(Ordering::Acquire) >= 4);
    assert_eq!(net::ACTIVE_REQUESTS.load(Ordering::Acquire), 0);
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
}
