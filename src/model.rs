use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Application configuration persisted to `config.json`.
pub struct Config {
    #[serde(rename = "currentUrl")]
    pub current_url: String,
    pub volume: u8,
    pub muted: bool,
    #[serde(rename = "lastPreset")]
    pub last_preset: usize,
    #[serde(rename = "lastBank", default)]
    pub last_bank: usize,

    #[serde(default)]
    pub presets: Vec<StationPreset>,

    #[serde(rename = "windowW")]
    pub window_w: u32,
    #[serde(rename = "windowH")]
    pub window_h: u32,
    #[serde(rename = "windowX")]
    pub window_x: i32,
    #[serde(rename = "windowY")]
    pub window_y: i32,
    #[serde(rename = "windowPosValid")]
    pub window_pos_valid: bool,

    #[serde(rename = "apiEndpoint")]
    pub api_endpoint: String,

    #[serde(rename = "customEqPresets")]
    pub custom_eq_presets: Vec<EqPreset>,

    #[serde(rename = "jsonPollIntervalSec", default)]
    pub json_poll_interval_sec: Option<u64>,
}

impl Config {
    /// Return a preset by index if it exists.
    pub fn preset(&self, index: usize) -> Option<&StationPreset> {
        self.presets.get(index)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// One radio station preset (slot with URL and EQ name).
pub struct StationPreset {
    pub name: String,
    pub url: String,
    pub eq: String,

    #[serde(rename = "metadataType")]
    pub metadata_type: String,

    #[serde(rename = "metadataUrl")]
    pub metadata_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Equalizer preset data (name + preamp + 10-band gains).
pub struct EqPreset {
    pub name: String,
    pub preamp: f32,
    pub bands: [f32; 10],
}

/// Insert or update a custom EQ preset by case-insensitive name.
pub fn upsert_custom_eq_preset(cfg: &mut Config, preset: EqPreset) {
    if let Some(existing) = cfg
        .custom_eq_presets
        .iter_mut()
        .find(|p| p.name.eq_ignore_ascii_case(&preset.name))
    {
        *existing = preset;
    } else {
        cfg.custom_eq_presets.push(preset);
    }
}

/// Delete a custom EQ preset and reset any stations that referenced it.
pub fn delete_custom_eq_preset(cfg: &mut Config, name: &str) {
    cfg.custom_eq_presets
        .retain(|p| !p.name.eq_ignore_ascii_case(name));

    for station in &mut cfg.presets {
        if station.eq.eq_ignore_ascii_case(name) {
            station.eq = "Off".to_string();
        }
    }
}
