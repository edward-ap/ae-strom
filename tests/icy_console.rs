#![cfg(feature = "live-audio")]
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct Fixture {
    url: String,
    stop: Arc<AtomicBool>,
    count: Arc<AtomicUsize>,
    worker: Option<thread::JoinHandle<()>>,
    dir: PathBuf,
}
fn serve(mut socket: TcpStream, number: usize, stop: Arc<AtomicBool>) {
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    socket
        .set_write_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut request = Vec::new();
    let mut one = [0u8];
    while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 && !stop.load(Ordering::Acquire) {
        match socket.read(&mut one) {
            Ok(0) => return,
            Ok(_) => request.push(one[0]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(_) => return,
        }
    }
    assert!(String::from_utf8_lossy(&request)
        .to_ascii_lowercase()
        .contains("icy-metadata: 1"));
    let header = format!("HTTP/1.1 200 OK\r\nConnection: close\r\nicy-name: Synthetic Radio\r\nicy-metaint: 16\r\nSessionId: synthetic-{number}\r\n\r\n");
    if socket.write_all(header.as_bytes()).is_err() {
        return;
    }
    let mut block = vec![0u8; 16];
    let raw = format!("StreamTitle='Track {number}';");
    let mut meta = raw.into_bytes();
    meta.resize(32, 0);
    block.push(2);
    block.extend(meta);
    while !stop.load(Ordering::Acquire) {
        // Fragmented reads must preserve exact block framing and raw text.
        for part in block.chunks(3) {
            if socket.write_all(part).is_err() {
                return;
            }
        }
        if number == 1 {
            return;
        } // Simulate a station's deliberate track-boundary EOF.
        thread::sleep(Duration::from_millis(100));
    }
}
impl Fixture {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}/icy", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let count = Arc::new(AtomicUsize::new(0));
        let stopped = stop.clone();
        let counts = count.clone();
        let worker = thread::spawn(move || {
            let mut clients = Vec::new();
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((socket, _)) => {
                        let number = counts.fetch_add(1, Ordering::AcqRel) + 1;
                        let stop = stopped.clone();
                        clients.push(thread::spawn(move || serve(socket, number, stop)));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
            for client in clients {
                client.join().unwrap();
            }
        });
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("strom-icy-console-{}-{id}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Self {
            url,
            stop,
            count,
            worker: Some(worker),
            dir,
        }
    }
    fn records(&self) -> Vec<Value> {
        std::fs::read_to_string(self.dir.join("capture.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
        std::fs::remove_dir_all(&self.dir).unwrap();
    }
}
fn finish(child: &mut std::process::Child) {
    let deadline = Instant::now() + Duration::from_secs(12);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "CLI failed: {status}");
            return;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("CLI failed to stop within its bound");
        }
        thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn legacy_peek_logs_headers_raw_blocks_eof_reconnect_and_clean_shutdown_without_audio() {
    let fixture = Fixture::new();
    let mut child = Command::new(env!("CARGO_BIN_EXE_icy_peek"))
        .arg(&fixture.url)
        .args(["--seconds", "4", "--log"])
        .arg(fixture.dir.join("capture.jsonl"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    finish(&mut child);
    let records = fixture.records();
    assert!(records.windows(2).all(
        |pair| pair[0]["elapsedMs"].as_u64().unwrap() <= pair[1]["elapsedMs"].as_u64().unwrap()
    ));
    let data: Vec<_> = records.iter().map(|r| &r["data"]).collect();
    assert!(fixture.count.load(Ordering::Acquire) >= 2);
    assert!(data
        .iter()
        .any(|d| d["event"] == "http_headers" && d["headers"].to_string().contains("synthetic-1")));
    assert!(data.iter().any(|d| d["event"] == "icy_block"
        && d["raw"] == "StreamTitle='Track 1';"
        && d["audioBytes"] == 16));
    assert!(data
        .iter()
        .any(|d| d["event"] == "http_end" && d["reason"] == "eof"));
    assert!(data
        .iter()
        .any(|d| d["event"] == "title" && d["title"] == "Track 2"));
    assert!(!data.iter().any(|d| d["event"] == "audio_connection"));
    let last = data.last().unwrap();
    assert_eq!(last["event"], "run_end");
    assert_eq!(last["reason"], "duration");
    for key in ["http", "metadata", "audio", "sessionOwners"] {
        assert_eq!(last["workers"][key], 0);
    }
    assert!(!fixture.dir.join("config.json").exists());
}
#[test]
fn manual_mark_fresh_listener_and_quit_leave_the_original_observer_running() {
    let fixture = Fixture::new();
    let log = fixture.dir.join("capture.jsonl");
    let mut child = Command::new(env!("CARGO_BIN_EXE_icy_peek"))
        .arg(&fixture.url)
        .args(["--seconds", "8", "--log"])
        .arg(&log)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"m different audible track\np\n")
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        let text = std::fs::read_to_string(&log).unwrap_or_default();
        if text
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .any(|r| r["data"]["event"] == "title" && r["data"]["role"] == "fresh-probe")
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Fresh probe did not produce a title");
        }
        thread::sleep(Duration::from_millis(20));
    }
    child.stdin.as_mut().unwrap().write_all(b"q\n").unwrap();
    finish(&mut child);
    let records = fixture.records();
    assert!(records
        .iter()
        .any(|r| r["data"]["event"] == "user_mark"
            && r["data"]["heard"] == "different audible track"));
    assert_eq!(
        records
            .iter()
            .filter(|r| r["data"]["event"] == "observer_start" && r["data"]["role"] == "observer")
            .count(),
        1
    );
    assert_eq!(records.last().unwrap()["data"]["reason"], "quit");
    for key in ["http", "metadata", "audio"] {
        assert_eq!(records.last().unwrap()["data"]["workers"][key], 0);
    }
}
