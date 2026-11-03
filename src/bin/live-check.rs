//! Explicit, bounded live backend acceptance tool. No URL/default data import is
//! baked in; the normal GUI still starts stopped, in fake mode unless enabled.
use ae_strom::controller::{Action, Controller, PlaybackMode, Update};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
fn action(controller: &Controller, action: Action) -> Result<(), String> {
    controller
        .submit(action)?
        .recv_timeout(Duration::from_secs(2))
        .map_err(|_| "Action timed out".to_owned())??;
    Ok(())
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut url, mut data, mut confirm) = (None, None, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--url" => url = args.next(),
            "--data-dir" => data = args.next().map(PathBuf::from),
            "--confirm-live" => confirm = true,
            _ => return Err(
                "Use --confirm-live --url <chosen HTTP(S) URL> --data-dir <new absolute directory>"
                    .into(),
            ),
        }
    }
    if !confirm {
        return Err("Live check requires explicit --confirm-live".into());
    }
    let url = url.ok_or("A user-selected --url is required")?;
    ae_strom::config::validate_stream_url(&url)?;
    let data = data.ok_or("An isolated --data-dir is required")?;
    if !data.is_absolute() || data.join("config.json").exists() {
        return Err("Use a new absolute data directory without an existing config.json".into());
    }
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let started = Instant::now();
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink_events = events.clone();
    let controller = Controller::start(
        data.join("config.json"),
        PlaybackMode::Live,
        Arc::new(move |update| {
            if let Update::State(state) = update {
                let mut records = sink_events.lock().unwrap();
                if records.len() < 128 {
                    records.push(json!({"elapsedMs":started.elapsed().as_millis(),"status":state.status,"generation":state.generation,"requestedPlaying":state.requested_playing,"error":state.error,"metadataPresent":state.full_title!="Connecting..."&&state.full_title!="Stopped"}));
                }
            }
        }),
    )?;
    let mut phases: Vec<Value> = Vec::new();
    let playback = (|| -> Result<(), String> {
        action(
            &controller,
            Action::SetStationUrl {
                bank: 0,
                slot: 0,
                url,
            },
        )?;
        action(&controller, Action::SetVolume { volume: 10 })?;
        action(&controller, Action::SetMuted { muted: false })?;
        for phase in 0..2 {
            action(&controller, Action::SetPlaying { playing: true })?;
            let begin = Instant::now();
            let diagnostic;
            loop {
                let d = controller.diagnostics()?;
                if d["active"] == true
                    && d["activeRevision"] == d["requestedRevision"]
                    && controller.snapshot().status == "playing"
                    && d["engine"]["outputCallbacks"].as_u64().unwrap_or(0) > 0
                    && d["engine"]["nonSilentSamples"].as_u64().unwrap_or(0) > 0
                {
                    diagnostic = d;
                    break;
                }
                if controller.snapshot().status == "error" {
                    return Err(controller
                        .snapshot()
                        .error
                        .unwrap_or("Terminal playback error".into()));
                }
                if begin.elapsed() > Duration::from_secs(10) {
                    return Err(
                        "No non-silent audio callbacks within the bounded test window".into(),
                    );
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            // Brief metadata opportunity, not a long station soak.
            std::thread::sleep(Duration::from_millis(750));
            let state = controller.snapshot();
            if state.status != "playing" || !state.requested_playing {
                return Err("Playback stopped or failed during the bounded observation".into());
            }
            phases.push(json!({"phase":phase+1,"connectedMs":begin.elapsed().as_millis(),"generation":state.generation,"status":state.status,"volume":state.volume,"muted":state.muted,"diagnostics":diagnostic,"metadataPresent":state.stations[0].name!=""||state.full_title!="Connecting..."}));
            action(&controller, Action::SetPlaying { playing: false })?;
            let stop = Instant::now();
            while controller.diagnostics()?["active"] == true {
                if stop.elapsed() > Duration::from_secs(2) {
                    return Err("Session did not retire within two seconds".into());
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let counts = ae_strom::session::worker_counts();
            if ["http", "metadata", "audio"]
                .iter()
                .any(|name| counts[*name] != 0)
            {
                return Err(format!("Workers remain after Stop: {counts}"));
            }
        }
        Ok(())
    })();
    let _ = action(&controller, Action::SetPlaying { playing: false });
    let shutdown = controller.shutdown();
    let workers = ae_strom::session::worker_counts();
    let pass = playback.is_ok()
        && shutdown.is_ok()
        && ["http", "metadata", "audio", "sessionOwners"]
            .iter()
            .all(|key| workers[*key] == 0);
    let record = json!({"pass":pass,"kind":"bounded user-authorized live backend check","elapsedMs":started.elapsed().as_millis(),"appVolume":10,"systemVolumeChanged":false,"deviceSelectionChanged":false,"userHeardSound":null,"phases":phases,"stateEvents":*events.lock().unwrap(),"workersAfterShutdown":workers,"playbackError":playback.err(),"shutdownError":shutdown.err()});
    std::fs::write(
        data.join("live-check.json"),
        serde_json::to_vec_pretty(&record).unwrap(),
    )
    .map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&record).unwrap());
    if pass {
        Ok(())
    } else {
        Err("Live backend check failed; see its isolated evidence".into())
    }
}
fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
