use crate::model::{Config, EqPreset, StationPreset};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

const BANK_COUNT: usize = 26;
const STATIONS_PER_BANK: usize = 10;

#[derive(Debug, thiserror::Error)]
/// Errors that can occur while loading or saving configuration.
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Read and normalize in memory only. Startup preparation owns backup/migration writes.
pub fn load_or_default(path: &Path) -> Result<Config, ConfigError> {
    if path.try_exists()? {
        decode_compatible(&fs::read_to_string(path)?)
    } else {
        Ok(default_config())
    }
}

pub(crate) fn document_value(raw: &str) -> Result<serde_json::Value, ConfigError> {
    crate::config_document::parse(raw)
}
pub(crate) fn stored_value(cfg: &Config) -> serde_json::Value {
    // Match the actual wire serializer: to_value expands f32 to f64 and can
    // falsely classify stable fractional EQ gains as a migration on every start.
    let bytes = serde_json::to_vec(&StoredConfig::from(cfg))
        .expect("Config contains JSON-compatible values");
    serde_json::from_slice(&bytes).expect("Serialized config is JSON")
}
/// Both historical layouts are accepted; unknown/duplicate/missing storage fields fail closed.
pub fn decode_compatible(raw: &str) -> Result<Config, ConfigError> {
    let value = document_value(raw)?;
    let mut cfg: Config = serde_json::from_value(value)?;
    if let Some(presets) = presets_from_station_banks(raw)? {
        cfg.presets = presets;
    }
    normalize_for_bank_layout(&mut cfg);
    preserve_legacy_reserved_eq(&mut cfg);
    for station in &mut cfg.presets {
        station.url = station.url.trim().to_owned();
        station.metadata_url = station.metadata_url.trim().to_owned();
    }
    validate(&cfg)?;
    Ok(cfg)
}

/// Saves a given configuration (`Config`) to a specified file path.
///
/// This function ensures that the parent directory of the specified path exists by
/// creating all directories in the path, if necessary. It then serializes the
/// configuration object into a pretty-printed JSON string and writes it to the specified
/// file.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` that specifies where to save the configuration file.
/// * `cfg` - A reference to the `Config` object that will be serialized and saved.
///
/// # Returns
///
/// * `Ok(())` - If the file is successfully saved without any errors.
/// * `Err(ConfigError)` - If an error occurs during directory creation, serialization, or writing the file.
///
/// # Errors
///
/// This function can fail for the following reasons:
/// * Failure to create the parent directory if it does not already exist.
/// * Failure to serialize the `Config` object into a JSON string.
/// * Failure to write the JSON string to the specified file path.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use ae_strom::config::{default_config, save};
///
/// let config = default_config();
/// let path = Path::new("config/settings.json");
///
/// match save(&path, &config) {
///     Ok(_) => println!("Configuration saved successfully."),
///     Err(e) => eprintln!("Failed to save configuration: {}", e),
/// }
/// ```
///
/// # Note
///
/// Ensure that the specified path is valid and the process has sufficient permissions to
/// create directories, serialize the configuration, and write to the file.
pub fn save(path: &Path, cfg: &Config) -> Result<(), ConfigError> {
    validate(cfg)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(&StoredConfig::from(cfg))?;
    static NEXT_WRITE: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let temporary = parent.join(format!(
        ".{}.{}-{}.tmp",
        name,
        std::process::id(),
        NEXT_WRITE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<(), ConfigError> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&json)?;
        file.sync_all()?;
        drop(file);
        // Same-directory replacement; the previous complete file survives a failed write.
        fs::rename(&temporary, path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub fn default_config() -> Config {
    Config {
        current_url: String::new(),
        volume: 50,
        muted: false,
        last_preset: 0,
        last_bank: 0,
        presets: default_presets(),
        window_w: 860,
        window_h: 32,
        window_x: 0,
        window_y: 0,
        window_pos_valid: false,
        api_endpoint: "https://de1.api.radio-browser.info".to_string(),
        custom_eq_presets: default_eq_presets(),
        json_poll_interval_sec: None,
    }
}

fn default_presets() -> Vec<StationPreset> {
    (0..10)
        .map(|_| StationPreset {
            name: String::new(),
            url: String::new(),
            eq: "Off".to_string(),
            metadata_type: "ICY".to_string(),
            metadata_url: String::new(),
        })
        .collect()
}

fn default_eq_presets() -> Vec<EqPreset> {
    Vec::new()
}

#[derive(Serialize)]
struct StoredConfig<'a> {
    #[serde(rename = "currentUrl")]
    current_url: &'a str,
    volume: u8,
    muted: bool,
    #[serde(rename = "lastPreset")]
    last_preset: usize,
    #[serde(rename = "lastBank")]
    last_bank: usize,
    #[serde(rename = "stationBanks")]
    station_banks: Vec<StoredBank<'a>>,
    #[serde(rename = "windowW")]
    window_w: u32,
    #[serde(rename = "windowH")]
    window_h: u32,
    #[serde(rename = "windowX")]
    window_x: i32,
    #[serde(rename = "windowY")]
    window_y: i32,
    #[serde(rename = "windowPosValid")]
    window_pos_valid: bool,
    #[serde(rename = "apiEndpoint")]
    api_endpoint: &'a str,
    #[serde(rename = "customEqPresets")]
    custom_eq_presets: &'a [EqPreset],
    #[serde(rename = "jsonPollIntervalSec")]
    json_poll_interval_sec: Option<u64>,
}

#[derive(Serialize)]
struct StoredBank<'a> {
    bank: String,
    slots: Vec<StoredSlot<'a>>,
}

#[derive(Serialize)]
struct StoredSlot<'a> {
    key: String,
    #[serde(flatten)]
    preset: &'a StationPreset,
}

#[derive(Deserialize)]
struct StoredBankOwned {
    bank: String,
    #[serde(default)]
    slots: Vec<StoredSlotOwned>,
}

#[derive(Deserialize)]
struct StoredSlotOwned {
    key: String,
    #[serde(flatten)]
    preset: StationPreset,
}

impl<'a> From<&'a Config> for StoredConfig<'a> {
    fn from(cfg: &'a Config) -> Self {
        Self {
            current_url: &cfg.current_url,
            volume: cfg.volume,
            muted: cfg.muted,
            last_preset: cfg.last_preset,
            last_bank: cfg.last_bank,
            station_banks: station_banks_from_flat(&cfg.presets),
            window_w: cfg.window_w,
            window_h: cfg.window_h,
            window_x: cfg.window_x,
            window_y: cfg.window_y,
            window_pos_valid: cfg.window_pos_valid,
            api_endpoint: &cfg.api_endpoint,
            custom_eq_presets: &cfg.custom_eq_presets,
            json_poll_interval_sec: cfg.json_poll_interval_sec,
        }
    }
}

fn station_banks_from_flat(presets: &[StationPreset]) -> Vec<StoredBank<'_>> {
    let mut banks = Vec::new();

    for bank_idx in 0..BANK_COUNT {
        let mut slots = Vec::new();
        for slot_idx in 0..STATIONS_PER_BANK {
            let abs_idx = bank_idx * STATIONS_PER_BANK + slot_idx;
            if let Some(preset) = presets.get(abs_idx) {
                if !is_empty_station(preset) {
                    slots.push(StoredSlot {
                        key: slot_key(slot_idx),
                        preset,
                    });
                }
            }
        }

        if !slots.is_empty() {
            banks.push(StoredBank {
                bank: bank_label(bank_idx),
                slots,
            });
        }
    }

    banks
}

fn presets_from_station_banks(json: &str) -> Result<Option<Vec<StationPreset>>, ConfigError> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let Some(banks_value) = value.get("stationBanks") else {
        return Ok(None);
    };
    let banks: Vec<StoredBankOwned> = serde_json::from_value(banks_value.clone())?;
    let mut presets = Vec::new();

    let mut seen = std::collections::HashSet::new();
    for bank in banks {
        if bank.bank.chars().count() != 1 {
            return Err(invalid("Invalid bank label"));
        }
        let bank_idx = parse_bank_label(&bank.bank).ok_or_else(|| invalid("Invalid bank label"))?;

        for slot in bank.slots {
            let slot_idx = parse_slot_key(&slot.key).ok_or_else(|| invalid("Invalid slot key"))?;
            let abs_idx = bank_idx * STATIONS_PER_BANK + slot_idx;
            if !seen.insert(abs_idx) {
                return Err(invalid("Duplicate bank/slot assignment"));
            }
            while presets.len() <= abs_idx {
                presets.push(empty_station_preset());
            }
            presets[abs_idx] = slot.preset;
        }
    }

    Ok(Some(presets))
}

fn is_empty_station(preset: &StationPreset) -> bool {
    preset.name.is_empty()
        && preset.url.is_empty()
        && (preset.eq.is_empty() || preset.eq == "Off")
        && (preset.metadata_type.is_empty() || preset.metadata_type == "ICY")
        && preset.metadata_url.is_empty()
}

fn empty_station_preset() -> StationPreset {
    StationPreset {
        name: String::new(),
        url: String::new(),
        eq: "Off".to_string(),
        metadata_type: String::new(),
        metadata_url: String::new(),
    }
}

fn bank_label(bank_idx: usize) -> String {
    ((b'A' + bank_idx as u8) as char).to_string()
}

fn parse_bank_label(label: &str) -> Option<usize> {
    let ch = label.chars().next()?.to_ascii_uppercase();
    if ('A'..='Z').contains(&ch) {
        Some((ch as u8 - b'A') as usize)
    } else {
        None
    }
}

fn slot_key(slot_idx: usize) -> String {
    match slot_idx {
        0..=8 => (slot_idx + 1).to_string(),
        9 => "0".to_string(),
        _ => "?".to_string(),
    }
}

fn parse_slot_key(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0),
        "2" => Some(1),
        "3" => Some(2),
        "4" => Some(3),
        "5" => Some(4),
        "6" => Some(5),
        "7" => Some(6),
        "8" => Some(7),
        "9" => Some(8),
        "0" => Some(9),
        _ => None,
    }
}

fn normalize_for_bank_layout(cfg: &mut Config) -> bool {
    let mut changed = false;

    if cfg.last_bank >= BANK_COUNT {
        cfg.last_bank = BANK_COUNT - 1;
        changed = true;
    }

    if cfg.last_preset >= STATIONS_PER_BANK {
        cfg.last_preset = STATIONS_PER_BANK - 1;
        changed = true;
    }

    while cfg.presets.len() < STATIONS_PER_BANK {
        cfg.presets.push(empty_station_preset());
        changed = true;
    }

    changed
}

/**
 * Test module for configuration round-trip functionality.
 *
 * The test verifies the ability to save a default configuration to a file and then
 * load it back, ensuring the written data and the loaded data match
 * for expected properties such as volume, presets, and custom equalizer presets.
 *
 * # Modules and Crates Used
 * - `super::*`: Imports all functions and types from the parent module.
 * - `std::env`: Used to fetch the system's temporary directory.
 * - `std::fs`: Used to handle file operations for removing temporary files.
 *
 * # Test Function: `config_roundtrip_default`
 * - **Purpose**: Ensures the `default_config` can be saved, loaded back,
 *   and verified for correctness.
 * - **Steps**:
 *   1. Create a temporary file path using the system's temp directory.
 *   2. Remove the file if it already exists to ensure a clean state.
 *   3. Generate a default configuration object using `default_config()`.
 *   4. Save the configuration to the file using `save()`.
 *   5. Load the configuration back from the file using `load_or_default()`.
 *   6. Verify that the properties of the loaded configuration (e.g., `volume`,
 *      number of presets, and custom EQ presets) match the original configuration.
 *   7. Clean up the temporary file after the test completes.
 *
 * # Assertions
 * - `assert_eq!` compares:
 *   1. `volume` of the original and loaded configuration.
 *   2. `presets` length of the original and loaded configuration.
 *   3. `custom_eq_presets` length of the original and loaded configuration.
 *
 * # Error Handling
 * - The `save` and `load_or_default` functions are expected to succeed,
 *   and any failure will result in a panic with an appropriate message.
 * - Temporary files are removed using `fs::remove_file` to ensure
 *   no leftover files in the temporary directory.
 */
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn config_roundtrip_default() {
        let mut path = env::temp_dir();
        path.push("mini_radio_test_config.json");

        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        let cfg = default_config();
        save(&path, &cfg).expect("save should succeed");
        let loaded = load_or_default(&path).expect("load should succeed");

        assert_eq!(loaded.volume, cfg.volume);
        assert_eq!(loaded.presets.len(), cfg.presets.len());
        assert_eq!(loaded.custom_eq_presets.len(), cfg.custom_eq_presets.len());

        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn old_config_without_bank_is_normalized_in_memory_without_rewriting() {
        let mut path = env::temp_dir();
        path.push("mini_radio_old_config_migration.json");

        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        let old_json = serde_json::json!({
            "currentUrl": "https://example.test/stream",
            "volume": 26,
            "muted": false,
            "lastPreset": 3,
            "presets": [
                {
                    "name": "Old Slot",
                    "url": "https://example.test/stream",
                    "eq": "Rock",
                    "metadataType": "ICY",
                    "metadataUrl": ""
                }
            ],
            "windowW": 860,
            "windowH": 32,
            "windowX": 0,
            "windowY": 0,
            "windowPosValid": false,
            "apiEndpoint": "https://de1.api.radio-browser.info",
            "customEqPresets": [],
            "jsonPollIntervalSec": null
        });

        fs::write(&path, serde_json::to_string_pretty(&old_json).unwrap()).unwrap();

        let loaded = load_or_default(&path).expect("old config should load");
        assert_eq!(loaded.last_bank, 0);
        assert_eq!(loaded.last_preset, 3);
        assert_eq!(loaded.presets.len(), 10);
        assert_eq!(loaded.presets[0].name, "Old Slot");

        let after: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(after, old_json);
        assert!(after.get("lastBank").is_none());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn sparse_bank_storage_does_not_write_empty_intermediate_slots() {
        let mut path = env::temp_dir();
        path.push("mini_radio_sparse_bank_storage.json");

        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        let mut cfg = default_config();
        cfg.last_bank = 25;
        cfg.last_preset = 9;
        let z_zero_idx = 25 * STATIONS_PER_BANK + 9;
        while cfg.presets.len() <= z_zero_idx {
            cfg.presets.push(empty_station_preset());
        }
        cfg.presets[z_zero_idx].url = "https://example.test/z0".to_string();
        cfg.presets[z_zero_idx].eq = "Metal".to_string();

        save(&path, &cfg).expect("save should succeed");

        let written: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(written.get("presets").is_none());
        let banks = written["stationBanks"].as_array().unwrap();
        assert_eq!(banks.len(), 1);
        assert_eq!(banks[0]["bank"], "Z");
        let slots = banks[0]["slots"].as_array().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0]["key"], "0");
        assert_eq!(slots[0]["url"], "https://example.test/z0");

        let loaded = load_or_default(&path).expect("compact config should load");
        assert_eq!(loaded.presets[z_zero_idx].url, "https://example.test/z0");
        assert_eq!(loaded.presets[z_zero_idx].eq, "Metal");

        fs::remove_file(&path).unwrap();
    }
}

/// Explicit import only. Parsing never writes or migrates the source file.
/// The destination belongs to this application's selected isolated data directory.
/// Preserve legacy custom EQ entries whose names collide with built-ins under
/// the new ASCII-insensitive identity. The old player resolved exact built-in
/// names first, then exact custom names. Preserve that binding when renaming.
fn preserve_legacy_reserved_eq(cfg: &mut Config) {
    let builtins = &*crate::presets::BUILTIN_EQ_PRESETS;
    let mut used = builtins
        .iter()
        .map(|p| p.name.to_ascii_lowercase())
        .chain(
            cfg.custom_eq_presets
                .iter()
                .map(|p| p.name.to_ascii_lowercase()),
        )
        .collect::<std::collections::HashSet<_>>();
    let mut next = 1usize;
    for preset in &mut cfg.custom_eq_presets {
        if !builtins
            .iter()
            .any(|b| b.name.eq_ignore_ascii_case(&preset.name))
        {
            continue;
        }
        let old = preset.name.clone();
        let new = loop {
            let candidate = format!("Imported EQ {next}");
            next += 1;
            if used.insert(candidate.to_ascii_lowercase()) {
                break candidate;
            }
        };
        // Exact built-in references already selected the built-in in Slint;
        // never redirect them to a formerly shadowed custom entry.
        if !builtins.iter().any(|b| b.name == old) {
            for station in &mut cfg.presets {
                if station.eq == old {
                    station.eq = new.clone();
                }
            }
        }
        preset.name = new;
    }
}

pub fn import_explicit(source: &Path, destination: &Path) -> Result<(), ConfigError> {
    if source == destination
        || (source.exists()
            && destination.exists()
            && fs::canonicalize(source)? == fs::canonicalize(destination)?)
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Import source and isolated destination must differ",
        )
        .into());
    }
    let raw = fs::read_to_string(source)?;
    let cfg = decode_compatible(&raw)?;
    if destination.try_exists()? {
        let previous = fs::read_to_string(destination)?;
        if stored_value(&decode_compatible(&previous)?) != stored_value(&cfg) {
            return Err(invalid("Import conflicts with existing config; use a fresh destination. Both files were preserved"));
        }
        // Import is idempotent and never rewrites an existing equivalent destination.
        return Ok(());
    }
    save(destination, &cfg)
}

#[cfg(test)]
mod isolated_storage_tests {
    use super::*;
    fn directory() -> std::path::PathBuf {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("radio-storage-{}-{id}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        path
    }
    #[test]
    fn atomic_replace_preserves_complete_schema_and_leaves_no_temporary_files() {
        let directory = directory();
        let path = directory.join("config.json");
        let mut cfg = default_config();
        save(&path, &cfg).unwrap();
        cfg.volume = 73;
        cfg.muted = true;
        save(&path, &cfg).unwrap();
        let loaded = load_or_default(&path).unwrap();
        assert_eq!(loaded.volume, 73);
        assert!(loaded.muted);
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(value.get("stationBanks").is_some());
        assert!(value.get("presets").is_none());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn explicit_legacy_import_never_migrates_source() {
        let directory = directory();
        let source = directory.join("selected-copy.json");
        let destination = directory.join("isolated/config.json");
        let cfg = default_config();
        let legacy = serde_json::to_vec_pretty(&cfg).unwrap();
        fs::write(&source, &legacy).unwrap();
        import_explicit(&source, &destination).unwrap();
        assert_eq!(fs::read(&source).unwrap(), legacy);
        assert_eq!(load_or_default(&destination).unwrap().volume, cfg.volume);
        assert!(import_explicit(&destination, &destination).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}

fn invalid(message: &str) -> ConfigError {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message).into()
}
pub fn validate_stream_url(value: &str) -> Result<(), String> {
    if value.len() > 16_384 || value.chars().any(char::is_whitespace) {
        return Err("Invalid stream URL length or whitespace".into());
    }
    let url = url::Url::parse(value).map_err(|_| "Invalid HTTP(S) stream URL".to_owned())?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none_or(str::is_empty) {
        return Err("Use an absolute HTTP(S) stream URL with a host".into());
    }
    Ok(())
}
/// Validation is shared by normal loading and explicit imports. No source file
/// is rewritten until the complete decoded configuration has passed this gate.
pub fn validate(cfg: &Config) -> Result<(), ConfigError> {
    if cfg.volume > 100 || cfg.last_bank >= 26 || cfg.last_preset >= 10 || cfg.presets.len() > 260 {
        return Err(invalid("Configuration values are out of range"));
    }
    let mut names = crate::presets::BUILTIN_EQ_PRESETS
        .iter()
        .map(|p| p.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    for preset in &cfg.custom_eq_presets {
        if preset.name.trim() != preset.name
            || preset.name.is_empty()
            || preset.name.len() > 128
            || preset.name.chars().any(char::is_control)
            || !names.insert(preset.name.to_ascii_lowercase())
        {
            return Err(invalid("Duplicate, reserved or invalid custom EQ name"));
        }
        if !std::iter::once(&preset.preamp)
            .chain(preset.bands.iter())
            .all(|v| v.is_finite() && (-15.0..=15.0).contains(v))
        {
            return Err(invalid("EQ values must be finite and within -15..15 dB"));
        }
    }
    for station in &cfg.presets {
        if !names.contains(&station.eq.to_ascii_lowercase()) {
            return Err(invalid("Station references an unknown EQ preset"));
        }
        for value in [&station.url, &station.metadata_url] {
            if !value.is_empty() {
                validate_stream_url(value)
                    .map_err(|_| invalid("Invalid station or metadata URL"))?;
            }
        }
        if !matches!(station.metadata_type.as_str(), "" | "ICY" | "JSON") {
            return Err(invalid("Unknown metadata strategy"));
        }
    }
    Ok(())
}

pub fn validate_custom_eq_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.len() > 128
        || name.trim() != name
        || name.chars().any(char::is_control)
        || crate::presets::BUILTIN_EQ_PRESETS
            .iter()
            .any(|p| p.name.eq_ignore_ascii_case(name))
    {
        Err("Choose a valid, non-reserved custom EQ name (max 128 bytes)".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod rejection_regressions {
    use super::*;
    #[test]
    fn invalid_import_and_save_leave_source_and_destination_unchanged() {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("radio-validation-{id}"));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("chosen.json");
        let destination = dir.join("config.json");
        let cfg = default_config();
        save(&destination, &cfg).unwrap();
        let before = fs::read(&destination).unwrap();
        for variation in 0..5 {
            let mut invalid = cfg.clone();
            match variation {
                0 => invalid.presets[0].url = "file:///not-a-stream".into(),
                1 => invalid.presets[0].eq = "Unknown".into(),
                2 => invalid.presets[0].metadata_type = "Unknown".into(),
                3 => invalid.custom_eq_presets.push(EqPreset {
                    name: "Bad\nName".into(),
                    preamp: 0.,
                    bands: [0.; 10],
                }),
                _ => invalid.custom_eq_presets.push(EqPreset {
                    name: "Custom".into(),
                    preamp: 16.,
                    bands: [0.; 10],
                }),
            }
            let bytes = serde_json::to_vec(&invalid).unwrap();
            fs::write(&source, &bytes).unwrap();
            assert!(import_explicit(&source, &destination).is_err());
            assert!(save(&destination, &invalid).is_err());
            assert_eq!(fs::read(&source).unwrap(), bytes);
            assert_eq!(fs::read(&destination).unwrap(), before);
        }
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn custom_name_validation_matches_ascii_case_resolution() {
        for name in ["", " Off", "Off", "oFF", "Bad\nName"] {
            assert!(validate_custom_eq_name(name).is_err());
        }
        let mut cfg = default_config();
        for name in ["Ä", "ä"] {
            assert!(validate_custom_eq_name(name).is_ok());
            cfg.custom_eq_presets.push(EqPreset {
                name: name.into(),
                preamp: 0.,
                bands: [0.; 10],
            });
        }
        validate(&cfg).unwrap();
        cfg.custom_eq_presets.push(EqPreset {
            name: "Ä".into(),
            preamp: 0.,
            bands: [0.; 10],
        });
        assert!(validate(&cfg).is_err());
    }
}

#[cfg(test)]
mod legacy_reserved_eq_tests {
    use super::*;
    #[test]
    fn explicit_import_preserves_shadowed_and_case_variant_custom_eq_and_bindings() {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("radio-reserved-eq-{id}"));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("selected.json");
        let target = dir.join("new/config.json");
        let mut cfg = default_config();
        cfg.custom_eq_presets = vec![
            EqPreset {
                name: "Rock".into(),
                preamp: 1.,
                bands: [1.; 10],
            },
            EqPreset {
                name: "rOCK".into(),
                preamp: 2.,
                bands: [2.; 10],
            },
            EqPreset {
                name: "Imported EQ 1".into(),
                preamp: 3.,
                bands: [3.; 10],
            },
            EqPreset {
                name: "Synthetic \u{0416} \"&".into(),
                preamp: -3.,
                bands: [-3.; 10],
            },
        ];
        cfg.presets[0].eq = "Rock".into(); // exact built-in won in the old player
        cfg.presets[1].eq = "rOCK".into(); // exact custom won in the old player
        cfg.presets[1].name = "Synthetic \u{0416} \" &".into();
        cfg.presets[1].url = "https://example.invalid/a?x=%D0%96&y=%22".into();
        let raw = serde_json::to_vec(&cfg).unwrap();
        fs::write(&source, &raw).unwrap();
        import_explicit(&source, &target).unwrap();
        let imported = load_or_default(&target).unwrap();
        assert_eq!(fs::read(&source).unwrap(), raw);
        assert_eq!(imported.custom_eq_presets.len(), 4);
        assert_eq!(imported.custom_eq_presets[0].name, "Imported EQ 2");
        assert_eq!(imported.custom_eq_presets[1].name, "Imported EQ 3");
        assert_eq!(imported.presets[0].eq, "Rock");
        assert_eq!(imported.presets[1].eq, "Imported EQ 3");
        assert_eq!(imported.presets[1].url, cfg.presets[1].url);
        assert_eq!(imported.presets[1].name, cfg.presets[1].name);
        for (a, b) in imported
            .custom_eq_presets
            .iter()
            .zip(&cfg.custom_eq_presets)
        {
            assert_eq!(a.preamp, b.preamp);
            assert_eq!(a.bands, b.bands);
        }
        save(&target, &imported).unwrap();
        let again = load_or_default(&target).unwrap();
        assert!(serde_json::to_value(&again).unwrap() == serde_json::to_value(&imported).unwrap());
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod legacy_url_whitespace_test {
    use super::*;
    #[test]
    fn import_trims_url_edges_only_and_preserves_source_special_characters() {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("radio-url-edges-{id}"));
        fs::create_dir_all(&dir).unwrap();
        let source = dir.join("chosen.json");
        let destination = dir.join("new/config.json");
        let mut cfg = default_config();
        cfg.presets = (0..260).map(|_| empty_station_preset()).collect();
        let slots = [5, 6, 7, 8, 9, 10, 259];
        // Seven stream URLs plus six metadata URLs: exactly 13 fields,
        // crossing a bank boundary and including the final Z/0 slot.
        for (n, &index) in slots.iter().enumerate() {
            cfg.presets[index].url = format!("  https://example.invalid/{n}/%D0%96?q=%22%20&x=1  ");
            cfg.presets[index].name = "Synthetic \u{0416} \" &".into();
            if n < 6 {
                cfg.presets[index].metadata_url =
                    format!("\t https://example.invalid/{n}/metadata?a=%26 \r\n");
            }
        }
        cfg.presets[0].url = "https://example.invalid/unchanged?a=%20".into();
        let raw = serde_json::to_vec(&cfg).unwrap();
        fs::write(&source, &raw).unwrap();
        import_explicit(&source, &destination).unwrap();
        let imported = load_or_default(&destination).unwrap();
        assert_eq!(fs::read(&source).unwrap(), raw);
        let mut changed_fields = 0;
        for (old, new) in cfg.presets.iter().zip(&imported.presets) {
            assert_eq!(new.url, old.url.trim());
            assert_eq!(new.metadata_url, old.metadata_url.trim());
            assert_eq!(new.name, old.name);
            assert_eq!(new.eq, old.eq);
            assert_eq!(new.metadata_type, old.metadata_type);
            changed_fields += usize::from(new.url != old.url);
            changed_fields += usize::from(new.metadata_url != old.metadata_url);
        }
        assert_eq!(imported.presets.len(), 260);
        assert_eq!(changed_fields, 13);
        assert_eq!(imported.presets[0].url, cfg.presets[0].url);
        assert!(validate_stream_url("https://example.invalid/internal space").is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
