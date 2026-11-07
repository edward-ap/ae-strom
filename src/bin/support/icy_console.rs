use ae_strom::metadata::{direct::watch_direct_icy, Info};
use log::{Level, LevelFilter, Log, Metadata, Record};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DEUTSCHROCK: &str = "https://s2-webradio.rockantenne.de/deutschrock/stream/aacp";
fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

struct Options {
    url: String,
    log: Option<PathBuf>,
    duration: Option<Duration>,
    help: bool,
}
impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            url: DEUTSCHROCK.into(),
            log: None,
            duration: None,
            help: false,
        };
        let mut args = args.into_iter();
        let mut url_set = false;
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => options.help = true,
                "--url" => {
                    if url_set {
                        return Err("Specify the URL only once".into());
                    }
                    options.url = args.next().ok_or("--url requires a URL")?;
                    url_set = true;
                }
                "--log" => {
                    options.log = Some(PathBuf::from(
                        args.next().ok_or("--log requires a file path")?,
                    ))
                }
                "--seconds" => {
                    let seconds: u64 = args
                        .next()
                        .ok_or("--seconds requires a number")?
                        .parse()
                        .map_err(|_| "Invalid --seconds")?;
                    if seconds == 0 {
                        return Err("--seconds must be positive".into());
                    }
                    options.duration = Some(Duration::from_secs(seconds));
                }
                value if !value.starts_with('-') && !url_set => {
                    options.url = value.into();
                    url_set = true;
                }
                _ => return Err(format!("Unknown or repeated argument: {arg}. Use --help.")),
            }
        }
        if !options.help {
            ae_strom::config::validate_stream_url(&options.url)?;
        }
        Ok(options)
    }
}

struct ConsoleLog {
    file: Mutex<File>,
    started: Instant,
    failed: AtomicBool,
}
impl Log for ConsoleLog {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target().starts_with("ae_strom")
            && (metadata.level() <= Level::Info || metadata.target() == "ae_strom::http_trace")
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let text = record.args().to_string();
        let data = serde_json::from_str::<Value>(&text)
            .unwrap_or_else(|_| json!({"event":"message","text":text}));
        let mut file = self.file.lock().unwrap();
        let elapsed_ms = self.started.elapsed().as_millis();
        let thread = thread::current();
        let entry = json!({"unixMs":unix_ms(),"elapsedMs":elapsed_ms,"level":record.level().as_str(),"target":record.target(),"thread":thread.name(),"data":data});
        let result = writeln!(file, "{entry}").and_then(|_| file.flush());
        if result.is_err() && !self.failed.swap(true, Ordering::AcqRel) {
            eprintln!("Cannot write the diagnostic log; stopping capture.");
        }
        // Serialize console/file emission together so their event order agrees.
        let _ = writeln!(
            io::stdout().lock(),
            "[+{:>9.3}s] {data}",
            elapsed_ms as f64 / 1000.0
        );
    }
    fn flush(&self) {
        let _ = self.file.lock().unwrap().flush();
    }
}
fn event(data: Value) {
    log::info!(target: "ae_strom::probe", "{data}");
}

struct IcyMonitor {
    cancel: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
    started: Instant,
}
impl IcyMonitor {
    fn start(
        url: String,
        role: &'static str,
        generation: u64,
        latest: Arc<Mutex<String>>,
        once: bool,
    ) -> Result<Self, String> {
        let cancel = Arc::new(AtomicBool::new(false));
        let stopped = cancel.clone();
        let worker = thread::Builder::new().name(format!("{role}-{generation}")).spawn(move || {
            event(json!({"event":"observer_start","role":role,"generation":generation}));
            let callback_stop = stopped.clone();
            let callback: Arc<dyn Fn(Info) + Send + Sync> = Arc::new(move |info| {
                if callback_stop.load(Ordering::Acquire) { return; }
                if !info.title.is_empty() {
                    *latest.lock().unwrap() = info.title.clone();
                    event(json!({"event":"title","role":role,"generation":generation,"title":info.title}));
                    if once { callback_stop.store(true, Ordering::Release); }
                }
                if let Some(name) = info.station_name { event(json!({"event":"station","role":role,"generation":generation,"name":name})); }
            });
            while !stopped.load(Ordering::Acquire) {
                match watch_direct_icy(&url, callback.clone(), stopped.clone()) {
                    Ok(worker) => { let _ = worker.join(); }
                    Err(error) => {
                        event(json!({"event":"observer_error","role":role,"generation":generation,"error":error.to_string()}));
                        let start = Instant::now();
                        while !stopped.load(Ordering::Acquire) && start.elapsed() < Duration::from_secs(2) { thread::sleep(Duration::from_millis(20)); }
                    }
                }
            }
            event(json!({"event":"observer_stop","role":role,"generation":generation}));
        }).map_err(|e| e.to_string())?;
        Ok(Self {
            cancel,
            worker: Some(worker),
            started: Instant::now(),
        })
    }
    fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(|w| w.is_finished())
    }
}
impl Drop for IcyMonitor {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn help() {
    println!("icy_peek.exe [URL] [--seconds N] [--log FILE]\nDefault URL: {DEUTSCHROCK}\nMetadata only: headers, raw ICY blocks and changed titles; no audio device.\nCommands (then Enter):\n  m [note]  record a note with the current ICY title\n  p         fresh ICY listener (up to 10 seconds), leaving the observer untouched\n  r         restart the ICY observer\n  q         stop and close the log");
}

pub fn run() -> Result<(), String> {
    let options = Options::parse(std::env::args().skip(1))?;
    if options.help {
        help();
        return Ok(());
    }
    let path = match options.log {
        Some(path) => path,
        None => std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .ok_or("No executable directory")?
            .join("local-data")
            .join(format!(
                "icy-peek-{}-{}.jsonl",
                unix_ms(),
                std::process::id()
            )),
    };
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| format!("Cannot create new log {}: {e}", path.display()))?;
    let logger = Box::leak(Box::new(ConsoleLog {
        file: Mutex::new(file),
        started: Instant::now(),
        failed: AtomicBool::new(false),
    }));
    log::set_logger(logger).map_err(|e| e.to_string())?;
    log::set_max_level(LevelFilter::Debug);
    println!("Log: {}", path.display());
    help();
    println!("Stream: {}", options.url);
    event(
        json!({"event":"run_start","url":options.url,"playback":false,"pid":std::process::id(),"topology":"ICY-only GET; no audio device, profile or UI"}),
    );
    let latest = Arc::new(Mutex::new(String::new()));
    let mut generation = 1;
    let mut observer = IcyMonitor::start(
        options.url.clone(),
        "observer",
        generation,
        latest.clone(),
        false,
    )?;
    let (input, commands) = mpsc::channel();
    thread::Builder::new()
        .name("console-input".into())
        .spawn(move || {
            for line in io::stdin().lock().lines() {
                match line {
                    Ok(line) => {
                        if input.send(Some(line)).is_err() {
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = input.send(None);
        })
        .map_err(|e| e.to_string())?;
    let mut probe: Option<IcyMonitor> = None;
    let mut probe_sequence = 0;
    let mut reason = "quit";
    loop {
        if logger.failed.load(Ordering::Acquire) {
            reason = "log_error";
            break;
        }
        if options
            .duration
            .is_some_and(|limit| logger.started.elapsed() >= limit)
        {
            reason = "duration";
            break;
        }
        if probe
            .as_ref()
            .is_some_and(|p| p.finished() || p.started.elapsed() >= Duration::from_secs(10))
        {
            event(
                json!({"event":"probe_end","result":if probe.as_ref().unwrap().finished() { "completed" } else { "timeout" }}),
            );
            drop(probe.take());
        }
        match commands.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(line)) => {
                let command = line.trim();
                let (name, rest) = command
                    .split_once(char::is_whitespace)
                    .unwrap_or((command, ""));
                match name {
                    "q" | "quit" => break,
                    "m" | "mark" => event(
                        json!({"event":"user_mark","heard":rest.trim(),"observerTitle":*latest.lock().unwrap()}),
                    ),
                    "r" => {
                        event(
                            json!({"event":"manual_restart","which":"observer","observerTitle":*latest.lock().unwrap()}),
                        );
                        drop(probe.take());
                        drop(observer);
                        generation += 1;
                        latest.lock().unwrap().clear();
                        observer = IcyMonitor::start(
                            options.url.clone(),
                            "observer",
                            generation,
                            latest.clone(),
                            false,
                        )?;
                    }
                    "p" => {
                        drop(probe.take());
                        probe_sequence += 1;
                        event(
                            json!({"event":"probe_request","sequence":probe_sequence,"observerTitle":*latest.lock().unwrap()}),
                        );
                        probe = Some(IcyMonitor::start(
                            options.url.clone(),
                            "fresh-probe",
                            probe_sequence,
                            Arc::new(Mutex::new(String::new())),
                            true,
                        )?);
                    }
                    "" => {}
                    _ => println!("Unknown command. Use m [note], p, r, q"),
                }
            }
            Ok(None) if options.duration.is_none() => {
                reason = "stdin_closed";
                break;
            }
            Ok(None) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                if options.duration.is_none() {
                    reason = "stdin_closed";
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
    drop(probe);
    drop(observer);
    event(json!({"event":"run_end","reason":reason,"workers":ae_strom::session::worker_counts()}));
    log::logger().flush();
    if logger.failed.load(Ordering::Acquire) {
        Err("Diagnostic log write failed".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(|a| a.to_string()))
    }
    #[test]
    fn defaults_and_legacy_positional_url() {
        assert_eq!(parse(&[]).unwrap().url, DEUTSCHROCK);
        let options = parse(&["http://127.0.0.1:1234/icy", "--seconds", "3"]).unwrap();
        assert_eq!(options.url, "http://127.0.0.1:1234/icy");
        assert_eq!(options.duration, Some(Duration::from_secs(3)));
    }
    #[test]
    fn rejects_bad_options_before_any_connection_or_audio() {
        for args in [
            vec!["--seconds", "0"],
            vec!["--volume", "101"],
            vec!["--url"],
            vec!["file:///test"],
            vec!["--bogus"],
            vec![DEUTSCHROCK, DEUTSCHROCK],
        ] {
            assert!(parse(&args).is_err());
        }
    }
}
