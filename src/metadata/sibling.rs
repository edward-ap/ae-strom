use crate::metadata::direct::{watch_direct_icy, DirectError};
use crate::metadata::model::StrategyHint;
use crate::metadata::Info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
/// Errors while probing sibling ICY streams.
pub enum SiblingError {
    NoFlacInUrl,
    AllCandidatesFailed,
}

impl std::fmt::Display for SiblingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SiblingError::NoFlacInUrl => write!(f, "no_flac_in_url"),
            SiblingError::AllCandidatesFailed => write!(f, "all_candidates_failed"),
        }
    }
}

/// Attempt to find ICY metadata by probing sibling streams.
///
/// When the main FLAC stream provides neither ICY nor usable container titles,
/// this function tries common alternative URLs (e.g., replacing "flac"
/// with "320", "128", "mp3", etc.) to find a stream that provides ICY metadata.
///
/// # Errors
/// Returns `SiblingError::NoFlacInUrl` if "flac" is not found in the URL,
/// or `SiblingError::AllCandidatesFailed` if no sibling stream provides ICY metadata.
pub fn watch_sibling_icy(
    main_url: &str,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(StrategyHint, std::thread::JoinHandle<()>), SiblingError> {
    if !main_url.to_lowercase().contains("flac") {
        return Err(SiblingError::NoFlacInUrl);
    }

    // Use warn level so messages are visible even when default filter is "warn".
    log::warn!(
        "sibling_icy: FLAC stream detected, probing ICY siblings for '{}'",
        main_url
    );

    // Try likely lower-bitrate siblings (including "k" suffixes) before generic fallbacks.
    let candidates = [
        "320", "320k", "256", "256k", "192", "192k", "128", "128k", "stream", "live", "aac",
        "aacp", "mp3",
    ];
    for cand in candidates {
        if stop_flag.load(Ordering::Acquire) {
            return Err(SiblingError::AllCandidatesFailed);
        }
        let candidate_url = replace_flac(main_url, cand);
        let cb: Arc<dyn Fn(Info) + Send + Sync> = {
            let on_info = on_info.clone();
            Arc::new(move |info: Info| {
                on_info(info);
            })
        };
        let res = watch_direct_icy(&candidate_url, cb, stop_flag.clone());

        match res {
            Ok(worker) => {
                log::warn!("sibling_icy: SUCCESS -> {}", candidate_url);
                return Ok((
                    StrategyHint {
                        r#type: "ICY".to_string(),
                        url: candidate_url,
                    },
                    worker,
                ));
            }
            Err(DirectError::NoIcyMetadata) => {
                log::warn!(
                    "sibling_icy: candidate '{}', url='{}', found=false",
                    cand,
                    candidate_url
                );
                continue;
            }
            Err(e) => {
                log::warn!(
                    "sibling_icy: error on candidate '{}' ({}): {:?} -> continuing",
                    cand,
                    candidate_url,
                    e
                );
                continue;
            }
        }
    }

    log::warn!("sibling_icy: no ICY metadata found among sibling candidates");
    Err(SiblingError::AllCandidatesFailed)
}

fn replace_flac(url: &str, replacement: &str) -> String {
    if let Some(idx) = url.to_lowercase().find("flac") {
        let mut result = String::new();
        result.push_str(&url[..idx]);
        result.push_str(replacement);
        result.push_str(&url[idx + 4..]);
        result
    } else {
        url.to_string()
    }
}
