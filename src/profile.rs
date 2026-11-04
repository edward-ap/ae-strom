//! Portable config selection and verified migration, with per-profile AppData WebView storage.
use crate::{config, model::Config};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

pub struct Prepared {
    pub config: Config,
    pub webview: PathBuf,
    pub origin: &'static str,
    _lock: File,
}
fn fail(message: &str) -> config::ConfigError {
    io::Error::new(io::ErrorKind::InvalidData, message).into()
}
fn next() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}
/// Stable non-secret identifier for cache/lock separation; not a security hash.
pub fn webview_path(app_local: &Path, directory: &Path) -> PathBuf {
    let path = directory.to_string_lossy();
    #[cfg(windows)]
    let path = path.to_lowercase();
    let mut hash = 0x6c62272e07bb014262b821756295c58du128;
    for byte in path.as_bytes() {
        hash ^= u128::from(*byte);
        hash = hash.wrapping_mul(0x1000000000000000000013b);
    }
    app_local
        .join("webview")
        .join(format!("profile-{hash:032x}"))
}
fn read(path: &Path) -> Result<Option<Vec<u8>>, config::ConfigError> {
    match fs::read(path) {
        Ok(raw) => Ok(Some(raw)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn decode(raw: &[u8]) -> Result<Config, config::ConfigError> {
    let text = std::str::from_utf8(raw).map_err(|_| fail("Configuration must be UTF-8"))?;
    config::decode_compatible(text)
}
fn equivalent(a: &Config, b: &Config) -> bool {
    // Controller pads to 26 banks: ignore only trailing wholly empty slots.
    fn normalized(cfg: &Config) -> serde_json::Value {
        config::stored_value(cfg)
    }
    normalized(a) == normalized(b)
}
pub(crate) fn backup(path: &Path, raw: &[u8]) -> Result<PathBuf, config::ConfigError> {
    let name = path
        .file_name()
        .ok_or_else(|| fail("Missing config filename"))?
        .to_string_lossy();
    let destination = path.with_file_name(format!("{name}.pre-migration-{}.bak", next()));
    let mut out = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&destination)?;
    out.write_all(raw)?;
    out.sync_all()?;
    drop(out);
    if fs::read(&destination)? != raw {
        return Err(fail(
            "Configuration backup verification failed; original was not changed",
        ));
    }
    Ok(destination)
}
fn writable(directory: &Path, path: &Path) -> Result<(), config::ConfigError> {
    if path.try_exists()? && fs::metadata(path)?.permissions().readonly() {
        return Err(fail(
            "Config is read-only; no alternate profile was substituted",
        ));
    }
    let probe = directory.join(format!(".ae-strom-write-probe-{}", next()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&probe)?;
    let result = (|| -> io::Result<()> {
        file.write_all(b"write probe")?;
        file.sync_all()
    })();
    drop(file);
    let remove = fs::remove_file(&probe);
    result?;
    remove?;
    Ok(())
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    legacy: PathBuf,
    backup_name: String,
}
fn acknowledged(
    receipt: &Path,
    legacy: &Path,
    raw: &[u8],
    directory: &Path,
) -> Result<bool, config::ConfigError> {
    let Some(bytes) = read(receipt)? else {
        return Ok(false);
    };
    let record: Receipt = serde_json::from_slice(&bytes)?;
    if record.legacy != fs::canonicalize(legacy)? {
        return Ok(false);
    }
    let name = Path::new(&record.backup_name);
    if name.file_name().is_none_or(|n| n != name.as_os_str())
        || !record.backup_name.starts_with("config.json.pre-migration-")
        || !record.backup_name.ends_with(".bak")
    {
        return Err(fail(
            "Invalid migration receipt; both configs were preserved",
        ));
    }
    Ok(read(&directory.join(name))?.as_deref() == Some(raw))
}
fn acknowledge(
    receipt: &Path,
    legacy: &Path,
    raw: &[u8],
    path: &Path,
) -> Result<(), config::ConfigError> {
    let backup = backup(path, raw)?;
    let record = Receipt {
        legacy: fs::canonicalize(legacy)?,
        backup_name: backup.file_name().unwrap().to_string_lossy().into_owned(),
    };
    let parent = receipt.parent().unwrap();
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".migration-{}.tmp", next()));
    let bytes = serde_json::to_vec_pretty(&record)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&temporary, receipt)?;
    if fs::read(receipt)? != bytes {
        return Err(fail(
            "Migration receipt verification failed; backup was retained",
        ));
    }
    Ok(())
}
pub fn prepare(
    path: &Path,
    legacy: Option<&Path>,
    import: Option<&Path>,
    app_local: &Path,
) -> Result<Prepared, config::ConfigError> {
    if !path.is_absolute() || !app_local.is_absolute() {
        return Err(fail("Profile and AppData paths must be absolute"));
    }
    let directory = path
        .parent()
        .ok_or_else(|| fail("Missing config directory"))?;
    fs::create_dir_all(directory)?;
    let canonical = fs::canonicalize(directory)?;
    let webview = webview_path(app_local, &canonical);
    let locks = app_local.join("profile-locks");
    fs::create_dir_all(&locks)?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(locks.join(format!(
            "{}.lock",
            webview.file_name().unwrap().to_string_lossy()
        )))?;
    lock.try_lock().map_err(|_| {
        fail("This profile is already open; close its other player before continuing")
    })?;
    let receipt = app_local.join("migrations").join(format!(
        "{}.json",
        webview.file_name().unwrap().to_string_lossy()
    ));
    let mut acknowledge_legacy = None;
    let existing = read(path)?;
    let existing_cfg = existing.as_deref().map(decode).transpose()?;
    let (selected_raw, selected, origin) = if let Some(source) = import {
        if source == path
            || (source.try_exists()?
                && path.try_exists()?
                && fs::canonicalize(source)? == fs::canonicalize(path)?)
        {
            return Err(fail("Import source and destination must differ"));
        }
        let raw = fs::read(source)?;
        let cfg = decode(&raw)?;
        if existing_cfg
            .as_ref()
            .is_some_and(|old| !equivalent(old, &cfg))
        {
            return Err(fail("Import conflicts with existing config.json; use a fresh --data-dir. Both files were preserved"));
        }
        (Some(raw), cfg, "explicit-import")
    } else if let Some(cfg) = existing_cfg {
        if let Some(old) = legacy {
            if let Some(raw) = read(old)? {
                if !acknowledged(&receipt, old, &raw, directory)? {
                    if !equivalent(&cfg, &decode(&raw)?) {
                        return Err(fail("Adjacent config.json and local-data/player/config.json differ. Choose one explicitly with --data-dir, or preserve/rename the adjacent file to migrate the previous profile. Both files were preserved"));
                    }
                    acknowledge_legacy = Some((old.to_path_buf(), raw));
                }
            }
        }
        (existing.clone(), cfg, "adjacent-or-explicit")
    } else if let Some(raw) = legacy.map(read).transpose()?.flatten() {
        let cfg = decode(&raw)?;
        acknowledge_legacy = Some((legacy.unwrap().to_path_buf(), raw.clone()));
        (Some(raw), cfg, "previous-profile")
    } else {
        (None, config::default_config(), "fresh")
    };
    fs::create_dir_all(&webview)?;
    writable(&webview, &webview.join(".profile-probe"))?;
    writable(directory, path)?;
    let needs_write = match existing.as_deref() {
        Some(raw) => {
            config::document_value(
                std::str::from_utf8(raw).map_err(|_| fail("Configuration must be UTF-8"))?,
            )? != config::stored_value(&selected)
        }
        None => true,
    };
    if let Some((old, raw)) = acknowledge_legacy.as_ref() {
        if read(old)?.as_deref() != Some(raw.as_slice()) {
            return Err(fail(
                "Previous profile changed during preparation; no migration was committed",
            ));
        }
        acknowledge(&receipt, old, raw, path)?;
    }
    if needs_write {
        // Check again after validation and before backup/commit. Never fall back to empties.
        if read(path)? != existing {
            return Err(fail(
                "Config changed during preparation; no migration was committed",
            ));
        }
        if let Some(raw) = existing.as_ref().or_else(|| {
            if acknowledge_legacy.is_none() {
                selected_raw.as_ref()
            } else {
                None
            }
        }) {
            backup(path, raw)?;
        }
        if read(path)? != existing {
            return Err(fail(
                "Config changed during backup; no migration was committed",
            ));
        }
        config::save(path, &selected)?;
        if !equivalent(&config::load_or_default(path)?, &selected) {
            return Err(fail(
                "Saved migration did not roundtrip; verified backup was retained",
            ));
        }
    }
    fs::create_dir_all(&webview)?;
    Ok(Prepared {
        config: selected,
        webview,
        origin,
        _lock: lock,
    })
}
