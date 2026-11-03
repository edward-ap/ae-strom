use crate::metadata::helpers::normalize_stream_title_case;
use crate::metadata::model::Info;
use crate::metadata::provider::spawn_metadata_worker;
use html_escape::decode_html_entities;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug)]
/// Errors returned while reading ICY metadata directly from the stream.
pub enum DirectError {
    Http,
    NoIcyMetadata,
    Io,
}

impl std::fmt::Display for DirectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectError::Http => write!(f, "http"),
            DirectError::NoIcyMetadata => write!(f, "no_icy_metadata"),
            DirectError::Io => write!(f, "io"),
        }
    }
}

/// Watch for ICY metadata directly from the stream.
///
/// Makes an HTTP request with Icy-MetaData header, reads metadata blocks
/// from the stream, and invokes the callback when titles change.
///
/// # Errors
/// Returns `DirectError::Http` if connection fails, `DirectError::NoIcyMetadata`
/// if the server doesn't support ICY metadata, or `DirectError::Io` on read errors.
pub fn watch_direct_icy(
    url: &str,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
    stop_flag: Arc<AtomicBool>,
) -> Result<std::thread::JoinHandle<()>, DirectError> {
    if stop_flag.load(Ordering::Acquire) {
        return Ok(std::thread::spawn(|| {}));
    }
    let agent = crate::net::get(url, stop_flag.clone()).set("Icy-MetaData", "1");
    let resp = agent.call().map_err(|_e| DirectError::Http)?;
    let station_name = resp.header("icy-name").and_then(normalize_station_name);

    let metaint: usize = resp
        .header("icy-metaint")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    if metaint == 0 || metaint > 1_048_576 {
        return Err(DirectError::NoIcyMetadata);
    }

    if let Some(name) = station_name.clone() {
        on_info(Info {
            title: String::new(),
            station_name: Some(name),
        });
    }

    let mut first = Some((resp.into_reader(), metaint));
    let url = url.to_owned();
    Ok(spawn_metadata_worker(
        "direct",
        stop_flag.clone(),
        move || {
            let (mut reader, metaint) = if let Some(first) = first.take() {
                first
            } else {
                let response = crate::net::get(&url, stop_flag.clone())
                    .set("Icy-MetaData", "1")
                    .call()
                    .map_err(|_| DirectError::Http)?;
                let metaint = response
                    .header("icy-metaint")
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|v| *v > 0 && *v <= 1_048_576)
                    .ok_or(DirectError::NoIcyMetadata)?;
                if let Some(name) = response.header("icy-name").and_then(normalize_station_name) {
                    on_info(Info {
                        title: String::new(),
                        station_name: Some(name),
                    });
                }
                (response.into_reader(), metaint)
            };
            direct_worker(&mut reader, metaint, on_info.clone(), stop_flag.clone())
        },
    ))
}

/// Worker that continuously reads ICY metadata blocks from the stream.
///
/// Reads audio chunks followed by metadata blocks, extracts StreamTitle,
/// and notifies on changes. Sends first result to confirm ICY works.
fn direct_worker(
    reader: &mut impl Read,
    metaint: usize,
    on_info: Arc<dyn Fn(Info) + Send + Sync>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), DirectError> {
    let mut audio_buf = vec![0u8; metaint];
    let mut meta_len_buf = [0u8; 1];
    let mut last_title = String::new();
    let mut _block_idx: u64 = 0;

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            return Ok(());
        }

        if let Err(_e) = reader.read_exact(&mut audio_buf) {
            return Err(DirectError::Io);
        }

        if let Err(_e) = reader.read_exact(&mut meta_len_buf) {
            return Err(DirectError::Io);
        }
        let meta_len = meta_len_buf[0] as usize * 16;
        if meta_len == 0 {
            continue;
        }

        let mut meta_buf = vec![0u8; meta_len];
        if let Err(_e) = reader.read_exact(&mut meta_buf) {
            return Err(DirectError::Io);
        }
        _block_idx += 1;

        if let Some(title) = parse_stream_title(&meta_buf) {
            if !title.is_empty() && title != last_title {
                on_info(Info {
                    title: title.clone(),
                    station_name: None,
                });
                last_title = title;
            }
        }
    }
}

/// Parse stream title from raw metadata bytes.
fn parse_stream_title(buf: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(buf);
    extract_stream_title(&s)
}

/// Extract StreamTitle value from ICY metadata string.
///
/// Handles single quotes, double quotes, or unquoted values.
fn extract_stream_title(meta: &str) -> Option<String> {
    let tail = meta.split_once("StreamTitle=")?.1.trim_start();
    if tail.is_empty() {
        return None;
    }

    let raw = match tail.chars().next() {
        Some('\'') => take_quoted_title(&tail[1..], '\''),
        Some('"') => take_quoted_title(&tail[1..], '"'),
        _ => take_unquoted_title(tail),
    };

    normalize_title(raw)
}

/// Extract quoted title value, handling both single and double quotes.
///
/// Icecast often sends `StreamTitle='Artist - Track';` (single quotes),
/// Shoutcast variants use double quotes, and some exotic servers drop quotes entirely.
fn take_quoted_title(s: &str, quote: char) -> &str {
    let bytes = s.as_bytes();
    let mut end: Option<usize> = None;
    for i in 0..bytes.len() {
        if bytes[i] != quote as u8 {
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        if j >= bytes.len() {
            end = Some(i);
            break;
        }
        if bytes[j] == b';' {
            let rest = &bytes[j + 1..];
            if rest.contains(&b'=') {
                end = Some(i);
                break;
            }
        }
    }
    let fallback = s.bytes().rposition(|b| b == quote as u8);
    let final_end = end.or(fallback).unwrap_or(s.len());
    &s[..final_end]
}

/// Extract unquoted title value up to semicolon or end of string.
fn take_unquoted_title(s: &str) -> &str {
    if let Some(pos) = s.find(';') {
        &s[..pos]
    } else {
        s
    }
}

/// Normalize and clean the extracted title.
///
/// Trims whitespace, decodes HTML entities, applies title case normalization,
/// and returns None if empty.
fn normalize_title(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let unescaped = decode_html_entities(trimmed).to_string();
    let normalized = normalize_stream_title_case(&unescaped);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_station_name(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(decode_html_entities(trimmed).to_string())
    }
}
