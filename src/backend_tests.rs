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
    request_log: Arc<Mutex<Vec<(String, bool)>>>,
    release_late_tags: Arc<AtomicBool>,
}
fn response(stream: &mut TcpStream, headers: &str, body: &[u8]) {
    let _ = stream
        .write_all(format!("HTTP/1.1 200 OK\r\nConnection: close\r\n{headers}\r\n").as_bytes());
    let _ = stream.write_all(body);
}
fn wave_header(frames: u32) -> Vec<u8> {
    wave_header_at_rate(frames, 48000)
}
fn wave_header_at_rate(frames: u32, rate: u32) -> Vec<u8> {
    let size = frames * 4;
    let mut out = Vec::new();
    out.extend(b"RIFF");
    out.extend((size + 36).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(2u16.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * 4).to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(16u16.to_le_bytes());
    out.extend(b"data");
    out.extend(size.to_le_bytes());
    out
}
fn comment_packet(fields: &[(&str, &str)]) -> Vec<u8> {
    let vendor = b"Strom synthetic fixture";
    let mut body = (vendor.len() as u32).to_le_bytes().to_vec();
    body.extend(vendor);
    body.extend((fields.len() as u32).to_le_bytes());
    for (key, value) in fields {
        let tag = format!("{key}={value}");
        body.extend((tag.len() as u32).to_le_bytes());
        body.extend(tag.as_bytes());
    }
    let mut packet = vec![
        0x84,
        ((body.len() >> 16) & 255) as u8,
        ((body.len() >> 8) & 255) as u8,
        (body.len() & 255) as u8,
    ];
    packet.extend(body);
    packet
}
fn tagged_flac(fields: &[(&str, &str)]) -> Vec<u8> {
    let plain = include_bytes!("../tests/fixtures/silence.flac");
    let mut out = plain[..42].to_vec();
    out[4] &= 0x7f;
    out.extend(comment_packet(fields));
    out.extend(&plain[42..]);
    out
}
fn ogg_page(serial: u32, sequence: u32, flags: u8, samples: u64, packet: &[u8]) -> Vec<u8> {
    let mut page = b"OggS".to_vec();
    page.extend([0, flags]);
    page.extend(samples.to_le_bytes());
    page.extend(serial.to_le_bytes());
    page.extend(sequence.to_le_bytes());
    page.extend([0; 4]);
    let mut lacing = vec![255; packet.len() / 255];
    lacing.push((packet.len() % 255) as u8);
    page.push(lacing.len() as u8);
    page.extend(lacing);
    page.extend(packet);
    let mut crc = 0u32;
    for &byte in &page {
        crc ^= (byte as u32) << 24;
        for _ in 0..8 {
            crc = (crc << 1) ^ if crc & 0x80000000 != 0 { 0x04c11db7 } else { 0 };
        }
    }
    page[22..26].copy_from_slice(&crc.to_le_bytes());
    page
}
fn chained_flac(first_title: bool, second_title: bool) -> Vec<u8> {
    let plain = include_bytes!("../tests/fixtures/silence.flac");
    let mut out = Vec::new();
    for link in 0..2u32 {
        let mut identification = b"\x7fFLAC\x01\x00\x00\x01fLaC".to_vec();
        let mut info = plain[4..42].to_vec();
        info[0] &= 0x7f;
        let word = u64::from_be_bytes(info[14..22].try_into().unwrap());
        info[14..22].copy_from_slice(&((word & !((1u64 << 36) - 1)) | (24 * 4096)).to_be_bytes());
        identification.extend(info);
        out.extend(ogg_page(1234 + link, 0, 2, 0, &identification));
        let fields = if link == 1 && second_title {
            vec![("ARTIST", "Second Artist"), ("TITLE", "Second Song")]
        } else if link == 0 && first_title {
            vec![("TITLE", "First Artist - First Song")]
        } else {
            vec![("ENCODER", "fixture")]
        };
        out.extend(ogg_page(1234 + link, 1, 0, 0, &comment_packet(&fields)));
        for frame in 0..24 {
            let packet = &plain[42 + frame * 14..42 + (frame + 1) * 14];
            out.extend(ogg_page(
                1234 + link,
                2 + frame as u32,
                if frame == 23 { 4 } else { 0 },
                ((frame + 1) * 4096) as u64,
                packet,
            ));
        }
    }
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
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let logged = request_log.clone();
        let release_late_tags = Arc::new(AtomicBool::new(false));
        let release_tags = release_late_tags.clone();
        let worker = thread::spawn(move || {
            let mut clients = Vec::new();
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let stop = stopped.clone();
                        let count = counts.clone();
                        let request_log = logged.clone();
                        let release_tags = release_tags.clone();
                        clients.push(thread::spawn(move||{
            stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();stream.set_write_timeout(Some(Duration::from_millis(100))).unwrap();
            let mut head=Vec::new();let mut one=[0u8;1];while head.len()<8192&&!head.ends_with(b"\r\n\r\n")&&!stop.load(Ordering::Acquire){match stream.read(&mut one){Ok(0)=>return,Ok(_)=>head.push(one[0]),Err(e)if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut)=>{},Err(_)=>return}}
            let request=String::from_utf8_lossy(&head);let path=request.split_whitespace().nth(1).unwrap_or("/");let number=count.fetch_add(1,Ordering::AcqRel);
            let icy_requested=request.to_ascii_lowercase().contains("icy-metadata: 1");
            let ordinal={let mut log=request_log.lock().unwrap();let n=log.iter().filter(|(p,_)|p==path).count()+1;log.push((path.to_owned(),icy_requested));n};
            match path {
                "/tagged.flac"|"/station-only.flac"|"/chained.flac"|"/late-tags.flac"|"/lost-tags.flac"|"/icy-tags.flac"=>{
                    let bytes=match path {
                        "/chained.flac"=>chained_flac(true,true),
                        "/late-tags.flac"=>chained_flac(false,true),
                        "/lost-tags.flac"=>chained_flac(true,false),
                        "/station-only.flac"=>tagged_flac(&[("STATION_NAME","Container Radio"),("ARTIST","Not A Station"),("ALBUM","Not A Track")]),
                        _=>tagged_flac(&[("ARTIST","Container Artist"),("TITLE","Container Song"),("STATION_NAME","Container Radio")]),
                    };
                    let embedded=path=="/icy-tags.flac";
                    let content=if path=="/chained.flac"||path=="/late-tags.flac"||path=="/lost-tags.flac"{"audio/ogg"}else{"audio/flac"};
                    response(&mut stream,&format!("Content-Type: {content}\r\n{}",if embedded{"icy-metaint: 64\r\n"}else{""}),&[]);
                    if path=="/late-tags.flac" {
                        // Hold the second logical stream until the test has actually
                        // observed the fallback title. Thread scheduling must not
                        // decide whether this exercises fallback retirement.
                        let split=bytes.windows(18).position(|page|page.starts_with(b"OggS")&&page[5]==2&&page[14..18]==1235u32.to_le_bytes()).unwrap();
                        if stream.write_all(&bytes[..split]).is_err(){return;}
                        while !stop.load(Ordering::Acquire)&&!release_tags.load(Ordering::Acquire){thread::sleep(Duration::from_millis(2));}
                        if !stop.load(Ordering::Acquire){let _=stream.write_all(&bytes[split..]);}
                        return;
                    }
                    for chunk in bytes.chunks(if embedded{64}else{257}) {
                        if stop.load(Ordering::Acquire)||stream.write_all(chunk).is_err(){return;}
                        if embedded&&chunk.len()==64 { let mut meta=b"StreamTitle='ICY Artist - ICY Song';".to_vec();meta.resize(48,0);if stream.write_all(&[3]).is_err()||stream.write_all(&meta).is_err(){return;} }
                        thread::sleep(Duration::from_millis(2));
                    }
                },
                "/pcmicy"|"/pcmicy-empty"|"/pcmicy-eof"|"/pcmflip"|"/pcmflipback"=>{
                    let embedded=match path {"/pcmflip"=>ordinal>1,"/pcmflipback"=>ordinal==1,_=>true};
                    let short=path=="/pcmicy-eof"||((path=="/pcmflip"||path=="/pcmflipback")&&ordinal==1);
                    let frames=if short{8192}else{480000};
                    let mut audio=wave_header(1_000_000);audio.extend(vec![0u8;frames*4]);
                    let extra=if embedded{"icy-metaint: 1024\r\nicy-name: Embedded Station\r\n"}else{""};
                    response(&mut stream,&format!("Content-Type: audio/wav\r\n{extra}"),&[]);
                    let raw=if path=="/pcmicy-empty"{String::new()}else{format!("StreamTitle='Session {ordinal}';")};let mut meta=raw.into_bytes();let length=meta.len().div_ceil(16);meta.resize(length*16,0);
                    for (index,block) in audio.chunks(1024).enumerate() {
                        if stop.load(Ordering::Acquire)||stream.write_all(block).is_err(){return;}
                        if embedded&&block.len()==1024 {
                            if stream.write_all(&[length as u8]).is_err(){return;}
                            if short&&index>=7 {let _=stream.write_all(&meta[..3]);return;}
                            if stream.write_all(&meta).is_err(){return;}
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                },
                "/stream.flac"=>{
                    let bytes=include_bytes!("../tests/fixtures/silence.flac");
                    response(&mut stream,&format!("Content-Type: audio/flac\r\nContent-Length: {}\r\n",bytes.len()),bytes);
                },
                "/adtsburst"|"/adtsicy"=>{
                    let extra=if path=="/adtsicy"{"icy-metaint: 300\r\n"}else{""};
                    response(&mut stream,&format!("Content-Type: audio/aac\r\nTransfer-Encoding: chunked\r\n{extra}"),&[]);
                    let frame=[0xff,0xf1,0x50,0x80,0x01,0xff,0xfc,0,0,0,0,0,0,0,0];
                    let mut chunk=frame.repeat(20);
                    if path=="/adtsicy" { chunk.push(1); chunk.extend(frame); chunk.push(0); }
                    for _ in 0..1000 {
                        if stop.load(Ordering::Acquire){break;}
                        if write!(stream,"{:x}\r\n",chunk.len()).is_err()||stream.write_all(&chunk).is_err()||stream.write_all(b"\r\n").is_err(){return;}
                        thread::sleep(Duration::from_millis(3));
                    }
                    let _=stream.write_all(b"0\r\n\r\n");
                },
                "/clock44100"=>{
                    let frames=132300u32;
                    response(&mut stream,&format!("Content-Type: audio/wav\r\nContent-Length: {}\r\n",frames*4+44),&wave_header_at_rate(frames,44100));
                    let started=Instant::now(); let block=vec![0u8;2048]; let mut written=0u32;
                    while written<frames&&!stop.load(Ordering::Acquire){
                        let count=(frames-written).min(512); if stream.write_all(&block[..count as usize*4]).is_err(){break;} written+=count;
                        let due=Duration::from_secs_f64(written as f64/44100.0);
                        while started.elapsed()<due&&!stop.load(Ordering::Acquire){thread::sleep(Duration::from_millis(1));}
                    }
                },
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
                "/unavailable"=>{let _=stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");},
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
            request_log,
            release_late_tags,
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
fn empty_bank_keeps_play_intent_and_configured_selections_resume() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    for bank in [0, 1] {
        action(
            &controller,
            Action::SetStationUrl {
                bank,
                slot: 0,
                url: server.path("/pcmicy"),
            },
        );
    }
    action(&controller, Action::SetVolume { volume: 17 });
    action(&controller, Action::SetMuted { muted: true });
    let playing = action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().status == "playing",
        Duration::from_secs(4),
    );
    let before = server.requests.load(Ordering::Acquire);
    let mut generation = playing.generation;
    for bank in [22, 25] {
        let empty = action(&controller, Action::SelectStation { bank, slot: 0 });
        assert!(empty.requested_playing);
        assert_eq!(empty.status, "stopped");
        assert_eq!(empty.full_title, "No stream URL");
        assert!(empty.error.is_none());
        assert!(empty.generation > generation);
        generation = empty.generation;
        wait(
            || {
                let workers = session::worker_counts();
                ["http", "metadata", "audio"]
                    .iter()
                    .all(|key| workers[*key] == 0)
            },
            Duration::from_secs(3),
        );
        let waiting = action(&controller, Action::SetVolume { volume: 17 });
        assert!(waiting.requested_playing && waiting.error.is_none());
        assert_eq!(server.requests.load(Ordering::Acquire), before);
    }
    for bank in [0, 1] {
        let before = server.requests.load(Ordering::Acquire);
        let resumed = action(&controller, Action::SelectStation { bank, slot: 0 });
        assert!(resumed.requested_playing && resumed.muted);
        assert_eq!(resumed.volume, 17);
        assert!(resumed.generation > generation);
        generation = resumed.generation;
        wait(
            || {
                controller.snapshot().status == "playing"
                    && server.requests.load(Ordering::Acquire) > before
            },
            Duration::from_secs(4),
        );
    }
    clean(controller, dir);
}

#[test]
fn empty_bank_stop_and_play_control_whether_configured_slots_resume() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/pcmicy"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().status == "playing",
        Duration::from_secs(4),
    );
    assert!(action(&controller, Action::SelectStation { bank: 22, slot: 0 }).requested_playing);
    wait(
        || {
            let workers = session::worker_counts();
            ["http", "metadata", "audio"]
                .iter()
                .all(|key| workers[*key] == 0)
        },
        Duration::from_secs(3),
    );
    let before = server.requests.load(Ordering::Acquire);
    let stopped = action(&controller, Action::SetPlaying { playing: false });
    assert!(!stopped.requested_playing);
    let selected = action(&controller, Action::SelectStation { bank: 0, slot: 0 });
    assert!(!selected.requested_playing);
    assert_eq!(selected.status, "stopped");
    assert_eq!(server.requests.load(Ordering::Acquire), before);
    action(&controller, Action::SelectStation { bank: 22, slot: 0 });
    let armed = action(&controller, Action::SetPlaying { playing: true });
    assert!(armed.requested_playing && armed.error.is_none());
    assert_eq!(armed.status, "stopped");
    assert_eq!(server.requests.load(Ordering::Acquire), before);
    assert!(action(&controller, Action::SelectStation { bank: 0, slot: 0 }).requested_playing);
    wait(
        || {
            controller.snapshot().status == "playing"
                && server.requests.load(Ordering::Acquire) > before
        },
        Duration::from_secs(4),
    );
    clean(controller, dir);
}

#[test]
fn http_ingress_clock_runs_before_consumer_and_accounts_queue_partial_read_and_cancel() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let cancel = Arc::new(AtomicBool::new(false));
    let response = net::get(&server.path("/adtsburst"), cancel)
        .with_buffer_timing()
        .call()
        .unwrap();
    let timing = response.buffer_timing().unwrap();
    let mut reader = response.into_reader();
    wait(
        || timing.snapshot().producer_waiting,
        Duration::from_secs(3),
    );
    let paused = timing.snapshot();
    assert_eq!(paused.delivered_bytes, 0);
    assert!(paused.received_adts.seconds > 0.0);
    assert!(paused.worker_pending_bytes > 0);
    assert_eq!(paused.channel_queued_chunks, 4);
    assert_eq!(paused.retained_bytes, paused.received_bytes);
    let mut one = [0u8; 1];
    reader.read_exact(&mut one).unwrap();
    let partial = timing.snapshot();
    assert_eq!(partial.delivered_bytes, 1);
    assert!(partial.reader_pending_bytes > 0);
    assert_eq!(
        partial.received_bytes - partial.delivered_bytes,
        partial.retained_bytes
    );
    assert_eq!(
        partial.retained_bytes,
        partial.worker_pending_bytes + partial.channel_queued_bytes + partial.reader_pending_bytes
    );
    drop(reader);
    let ended = timing.snapshot();
    assert!(ended.producer_ended && ended.reader_closed);
    assert_eq!(ended.retained_bytes, 0);
    assert_eq!(
        ended.received_bytes,
        ended.delivered_bytes + ended.discarded_bytes
    );
    assert_eq!(net::ACTIVE_REQUESTS.load(Ordering::Acquire), 0);
    let response = net::get(&server.path("/adtsicy"), Arc::new(AtomicBool::new(false)))
        .with_buffer_timing()
        .call()
        .unwrap();
    let other = response.buffer_timing().unwrap();
    let mut reader = response.into_reader();
    reader.read_exact(&mut one).unwrap();
    let framed = other.snapshot();
    assert_ne!(framed.request_id, ended.request_id);
    assert!(framed.enabled);
    assert!(framed.received_adts.enabled);
    assert_eq!(
        framed.received_adts.frames,
        framed.received_bytes / 317 * 20 + (framed.received_bytes % 317).min(300) / 15,
        "the valid-looking ADTS frame inside each ICY block must be excluded"
    );
    assert_eq!(framed.received_adts.discarded_bytes, 0);
    assert_eq!(framed.received_adts.sync_losses, 0);
    drop(reader);
}

#[test]
fn independent_media_clocks_survive_resampling_and_follow_paced_loopback_without_device() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let engine = crate::audio::AudioEngine::new_sink();
    engine.play_url(server.path("/clock44100"));
    wait(
        || engine.diagnostics().timing.played_audio_seconds >= 1.5,
        Duration::from_secs(5),
    );
    let t = engine.diagnostics().timing;
    assert!(t.active);
    assert_eq!(t.declared_sample_rate, 44100);
    assert_eq!(t.decoded_sample_rate, 44100);
    assert_eq!(t.output_sample_rate, 48000);
    assert_eq!(t.output_channels, 2);
    assert_eq!(t.packet_duration_unknown, 0);
    assert_eq!(t.packet_discontinuities, 0);
    assert!((t.source_packet_seconds - t.decoded_audio_seconds).abs() < 0.03);
    assert!((t.decoded_audio_seconds - t.generated_output_seconds).abs() < 0.03);
    assert!((t.generated_output_seconds - t.played_audio_seconds).abs() < 0.03);
    assert!((t.source_packet_seconds / t.wall_seconds - 1.0).abs() < 0.15);
    assert_eq!(t.underrun_samples, 0);
    assert!(t.http.enabled);
    assert_eq!(t.http.request_id, t.request_id);
    assert!(t.http.received_bytes >= t.http.delivered_bytes);
    assert_eq!(
        t.http.retained_bytes,
        t.http.received_bytes - t.http.delivered_bytes - t.http.discarded_bytes
    );
    engine.stop();
    let stopped = engine.diagnostics().timing;
    assert!(!stopped.active);
    let frozen = stopped.wall_seconds;
    thread::sleep(Duration::from_millis(20));
    assert_eq!(engine.diagnostics().timing.wall_seconds, frozen);
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
fn transient_start_failure_reports_http_status_and_recovers_on_selection() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    for (slot, path) in [(0, "/unavailable"), (1, "/pcmicy")] {
        action(
            &controller,
            Action::SetStationUrl {
                bank: 0,
                slot,
                url: server.path(path),
            },
        );
    }
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().status == "reconnecting",
        Duration::from_secs(4),
    );
    let failed = controller.snapshot();
    assert!(failed.requested_playing);
    let reason = failed.error.as_deref().unwrap();
    assert!(
        reason.contains("502"),
        "specific HTTP cause was lost: {reason}"
    );
    assert!(reason.contains("retrying"));
    assert!(!reason.contains("Stream or output interrupted"));
    let resumed = action(&controller, Action::SelectStation { bank: 0, slot: 1 });
    assert!(resumed.generation > failed.generation);
    wait(
        || controller.snapshot().status == "playing",
        Duration::from_secs(4),
    );
    assert!(controller.snapshot().error.is_none());
    clean(controller, dir);
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

fn play_routed(
    service: &session::LivePlayback,
    url: String,
    hint: StrategyHint,
    titles: Arc<Mutex<Vec<String>>>,
) -> Arc<Mutex<Vec<StrategyHint>>> {
    let strategies = Arc::new(Mutex::new(Vec::new()));
    let resolved = strategies.clone();
    service.play(session::SessionSpec {
        url,
        hint,
        poll: Duration::from_secs(5),
        cancel: Arc::new(AtomicBool::new(false)),
        on_info: Arc::new(move |info, _| {
            if !info.title.is_empty() {
                titles.lock().unwrap().push(info.title);
            }
        }),
        on_strategy: Arc::new(move |hint| resolved.lock().unwrap().push(hint)),
    });
    strategies
}
fn stop_routed(service: &mut session::LivePlayback) {
    service.shutdown();
    assert_eq!(
        session::worker_counts(),
        serde_json::json!({"http":0,"metadata":0,"audio":0,"sessionOwners":0})
    );
}

#[test]
fn audio_icy_uses_one_connection_and_overrides_even_an_explicit_json_hint() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    let strategies = play_routed(
        &service,
        server.path("/pcmicy"),
        StrategyHint {
            r#type: "JSON".into(),
            url: server.path("/json"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles.lock().unwrap().iter().any(|s| s == "Session 1")
                && service.diagnostics().engine.output_samples > 5000
        },
        Duration::from_secs(4),
    );
    assert_eq!(service.diagnostics().engine.timing.decode_error_packets, 0);
    assert!(
        strategies.lock().unwrap().is_empty(),
        "in-band ICY must not overwrite the configured fallback hint"
    );
    assert_eq!(
        *server.request_log.lock().unwrap(),
        vec![("/pcmicy".into(), true)]
    );
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
    stop_routed(&mut service);
}

#[test]
fn advertised_icy_without_titles_never_opens_a_parallel_metadata_connection() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/pcmicy-empty"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || service.diagnostics().engine.output_samples > 15000,
        Duration::from_secs(4),
    );
    assert!(titles.lock().unwrap().is_empty());
    assert_eq!(
        *server.request_log.lock().unwrap(),
        vec![("/pcmicy-empty".into(), true)]
    );
    stop_routed(&mut service);
}

#[test]
fn actual_flac_without_icy_uses_configured_sibling_without_reopening_audio() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/stream.flac"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles.lock().unwrap().iter().any(|s| s == "Second")
                && service.diagnostics().engine.output_samples > 5000
        },
        Duration::from_secs(4),
    );
    let diag = service.diagnostics().engine;
    assert_eq!(diag.timing.codec, "flac");
    assert_eq!(diag.timing.decoded_sample_rate, 48000);
    assert!(diag.decoded_samples > 0);
    let requests = server.request_log.lock().unwrap().clone();
    assert_eq!(
        requests.iter().filter(|(p, _)| p == "/stream.flac").count(),
        1
    );
    assert_eq!(requests.iter().filter(|(p, _)| p == "/icy").count(), 1);
    assert_eq!(requests.len(), 2);
    stop_routed(&mut service);
}

#[test]
fn partial_audio_icy_block_reconnects_audio_and_metadata_together() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/pcmicy-eof"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || titles.lock().unwrap().iter().any(|s| s == "Session 2"),
        Duration::from_secs(5),
    );
    let requests = server.request_log.lock().unwrap().clone();
    assert!(requests.len() >= 2);
    assert!(requests
        .iter()
        .all(|(p, header)| p == "/pcmicy-eof" && *header));
    assert!(titles.lock().unwrap().iter().any(|s| s == "Session 1"));
    stop_routed(&mut service);
}

#[test]
fn audio_reconnect_with_icy_stops_fallback_and_keeps_session_cancellation_separate() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/pcmflip"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || titles.lock().unwrap().iter().any(|s| s == "Session 2"),
        Duration::from_secs(5),
    );
    thread::sleep(Duration::from_millis(150));
    let seen = titles.lock().unwrap().clone();
    assert!(seen.iter().any(|s| s == "Second"));
    let switch = seen.iter().position(|s| s == "Session 2").unwrap();
    assert!(seen[switch..].iter().all(|s| s != "Second"));
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
    let requests = server.request_log.lock().unwrap().clone();
    assert_eq!(requests.iter().filter(|(p, _)| p == "/pcmflip").count(), 2);
    assert_eq!(requests.iter().filter(|(p, _)| p == "/icy").count(), 1);
    assert_eq!(requests.len(), 3);
    stop_routed(&mut service);
}

#[test]
fn audio_reconnect_without_icy_starts_the_configured_fallback() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/pcmflipback"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles.lock().unwrap().iter().any(|s| s == "Second")
                && service.diagnostics().engine.connections >= 2
        },
        Duration::from_secs(5),
    );
    assert!(titles.lock().unwrap().iter().any(|s| s == "Session 1"));
    let requests = server.request_log.lock().unwrap().clone();
    assert_eq!(
        requests.iter().filter(|(p, _)| p == "/pcmflipback").count(),
        2
    );
    assert_eq!(requests.iter().filter(|(p, _)| p == "/icy").count(), 1);
    assert_eq!(requests.len(), 3);
    stop_routed(&mut service);
}

#[test]
fn native_flac_tags_supply_title_and_fill_an_empty_station_without_fallback() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let (controller, dir) = actor();
    action(
        &controller,
        Action::SetStationUrl {
            bank: 0,
            slot: 0,
            url: server.path("/tagged.flac"),
        },
    );
    action(&controller, Action::SetPlaying { playing: true });
    wait(
        || controller.snapshot().full_title == "Container Artist - Container Song",
        Duration::from_secs(4),
    );
    assert_eq!(controller.snapshot().stations[0].name, "Container Radio");
    assert_eq!(
        *server.request_log.lock().unwrap(),
        vec![("/tagged.flac".into(), true)]
    );
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
    clean(controller, dir);
}

#[test]
fn ogg_flac_chain_updates_title_on_the_same_http_connection() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    let strategies = play_routed(
        &service,
        server.path("/chained.flac"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles
                .lock()
                .unwrap()
                .iter()
                .any(|t| t == "Second Artist - Second Song")
        },
        Duration::from_secs(8),
    );
    assert!(titles
        .lock()
        .unwrap()
        .iter()
        .any(|t| t == "First Artist - First Song"));
    assert_eq!(
        *server.request_log.lock().unwrap(),
        vec![("/chained.flac".into(), true)]
    );
    assert!(strategies.lock().unwrap().is_empty());
    assert_eq!(service.diagnostics().engine.connections, 1);
    assert_eq!(service.diagnostics().engine.timing.decode_error_packets, 0);
    stop_routed(&mut service);
}

#[test]
fn late_container_title_stops_external_fallback_without_restarting_audio() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/late-tags.flac"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || titles.lock().unwrap().iter().any(|t| t == "Second"),
        Duration::from_secs(4),
    );
    server.release_late_tags.store(true, Ordering::Release);
    wait(
        || {
            titles
                .lock()
                .unwrap()
                .iter()
                .any(|t| t == "Second Artist - Second Song")
        },
        Duration::from_secs(8),
    );
    let seen = titles.lock().unwrap().clone();
    assert!(seen.iter().any(|t| t == "Second"));
    assert_eq!(*seen.last().unwrap(), "Second Artist - Second Song");
    assert_eq!(
        crate::metadata::provider::ACTIVE_METADATA_WORKERS.load(Ordering::Acquire),
        0
    );
    assert_eq!(server.request_log.lock().unwrap().len(), 2);
    assert_eq!(service.diagnostics().engine.connections, 1);
    stop_routed(&mut service);
}

#[test]
fn icy_titles_take_priority_over_real_flac_container_tags() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/icy-tags.flac"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles
                .lock()
                .unwrap()
                .iter()
                .any(|t| t == "Icy Artist - Icy Song")
                && service.diagnostics().engine.output_samples > 5000
        },
        Duration::from_secs(4),
    );
    assert!(titles
        .lock()
        .unwrap()
        .iter()
        .all(|t| t == "Icy Artist - Icy Song"));
    assert_eq!(
        *server.request_log.lock().unwrap(),
        vec![("/icy-tags.flac".into(), true)]
    );
    stop_routed(&mut service);
}

#[test]
fn station_only_container_metadata_does_not_suppress_track_fallback() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/station-only.flac"),
        StrategyHint {
            r#type: "JSON".into(),
            url: server.path("/json"),
        },
        titles.clone(),
    );
    wait(
        || {
            titles
                .lock()
                .unwrap()
                .iter()
                .any(|t| t == "Fixture Artist - Json Track")
        },
        Duration::from_secs(4),
    );
    let requests = server.request_log.lock().unwrap().clone();
    assert_eq!(
        requests
            .iter()
            .filter(|(path, _)| path == "/station-only.flac")
            .count(),
        1
    );
    // JSON polling may already have fetched again when the first title arrives.
    // The contract here is one audio GET and a working JSON-only fallback.
    assert!(requests.iter().any(|(path, _)| path == "/json"));
    assert!(requests
        .iter()
        .all(|(path, _)| matches!(path.as_str(), "/station-only.flac" | "/json")));
    assert!(titles.lock().unwrap().iter().all(|t| t != "Not A Station"));
    stop_routed(&mut service);
}

#[test]
fn container_title_loss_on_a_new_ogg_link_enables_fallback() {
    let _serial = SERIAL.lock().unwrap();
    let server = Server::new();
    let mut service = session::LivePlayback::new_sink();
    let titles = Arc::new(Mutex::new(Vec::new()));
    play_routed(
        &service,
        server.path("/lost-tags.flac"),
        StrategyHint {
            r#type: "ICY".into(),
            url: server.path("/icy"),
        },
        titles.clone(),
    );
    wait(
        || titles.lock().unwrap().iter().any(|t| t == "Second"),
        Duration::from_secs(8),
    );
    assert!(titles
        .lock()
        .unwrap()
        .iter()
        .any(|t| t == "First Artist - First Song"));
    assert_eq!(server.request_log.lock().unwrap().len(), 2);
    assert_eq!(service.diagnostics().engine.connections, 1);
    stop_routed(&mut service);
}
