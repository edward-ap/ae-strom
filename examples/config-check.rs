//! Read-only offline config diagnostics. No profile preparation, writes, network or audio.
use ae_strom::{config, startup_failure::Failure};
fn main() {
    let Some(path) = std::env::args_os().nth(1).map(std::path::PathBuf::from) else {
        eprintln!("Use: config-check <absolute config file path>");
        std::process::exit(2);
    };
    if !path.is_absolute() {
        eprintln!("Use an absolute file path. No file was read or changed.");
        std::process::exit(2);
    }
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("{}", Failure::profile(error.into()));
            std::process::exit(1);
        }
    };
    let raw = match std::str::from_utf8(&bytes) {
        Ok(raw) => raw,
        Err(_) => {
            eprintln!("Field: $\nReason: configuration must be UTF-8. No file was changed.");
            std::process::exit(1);
        }
    };
    let result = config::decode_compatible(raw);
    if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
        eprintln!("Source changed while being checked. No file was written by this tool.");
        std::process::exit(1);
    }
    match result {
        Ok(cfg) => println!(
            "{}",
            serde_json::json!({"pass":true,"stations":cfg.presets.iter().filter(|s| !s.url.is_empty()).count(),"customEqPresets":cfg.custom_eq_presets.len(),"sourceUnchanged":true,"networkRequests":0,"audio":false,"acceptedLegacyCases":{"urlFieldsWithLiteralSpaces":cfg.presets.iter().flat_map(|s| [&s.url,&s.metadata_url]).filter(|u| u.contains(' ')).count(),"longExistingEqNames":cfg.custom_eq_presets.iter().filter(|p| p.name.len()>128).count(),"paddedExistingEqNames":cfg.custom_eq_presets.iter().filter(|p| p.name.trim()!=p.name).count()}})
        ),
        Err(error) => {
            eprintln!("{}", Failure::profile(error));
            std::process::exit(1);
        }
    }
}
