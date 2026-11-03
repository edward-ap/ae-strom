//! Offline schema and lossless save/load check for an explicitly selected COPY.
//! Never pass the original profile: loading may normalize the selected copy.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(std::path::PathBuf::from)
        .ok_or("Pass an absolute copied config path")?;
    if !path.is_absolute() {
        return Err("Use an absolute copied config path".into());
    }
    let original: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    let cfg = ae_strom::config::load_or_default(&path)?;
    ae_strom::config::validate(&cfg)?;
    let roundtrip = path.with_file_name("roundtrip-config.json");
    if roundtrip.exists() {
        return Err("Roundtrip output already exists; choose a fresh audit copy".into());
    }
    ae_strom::config::save(&roundtrip, &cfg)?;
    let actual: serde_json::Value = serde_json::from_slice(&std::fs::read(&roundtrip)?)?;
    if original != actual {
        return Err(
            "Roundtrip changed configuration data; review privately before transfer".into(),
        );
    }
    let loaded = ae_strom::config::load_or_default(&roundtrip)?;
    if serde_json::to_value(&cfg)? != serde_json::to_value(&loaded)? {
        return Err("Reload differs from validated configuration".into());
    }
    let station_count = cfg.presets.iter().filter(|s| !s.url.is_empty()).count();
    let banks = cfg
        .presets
        .chunks(10)
        .filter(|bank| bank.iter().any(|s| !s.url.is_empty()))
        .count();
    println!(
        "{}",
        serde_json::json!({"pass":true,"stations":station_count,"populatedBanks":banks,"customEqPresets":cfg.custom_eq_presets.len(),"allFieldsRoundtripEqual":true,"networkRequests":0})
    );
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!("Profile verification failed; no source profile was modified. Inspect only the explicitly selected audit copy.");
        std::process::exit(1);
    }
}
