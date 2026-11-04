use crate::{
    config,
    model::{Config, EqPreset},
    profile,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
struct Fixture {
    root: PathBuf,
    path: PathBuf,
    legacy: PathBuf,
    app: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "ae-profile-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("local-data/player")).unwrap();
        Self {
            path: root.join("config.json"),
            legacy: root.join("local-data/player/config.json"),
            app: root.join("appdata"),
            root,
        }
    }
    fn prepare(&self) -> Result<profile::Prepared, config::ConfigError> {
        profile::prepare(&self.path, Some(&self.legacy), None, &self.app)
    }
    fn backups(&self) -> Vec<PathBuf> {
        fs::read_dir(&self.root)
            .unwrap()
            .map(|p| p.unwrap().path())
            .filter(|p| p.extension().is_some_and(|s| s == "bak"))
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn station() -> Config {
    let mut c = config::default_config();
    c.presets[0].name = "Synthetic Ã† Ð– &".into();
    c.presets[0].url = "https://synthetic.invalid/a?x=%22%20&y=1".into();
    c
}
fn raw(path: &Path, cfg: &Config) -> Vec<u8> {
    let bytes = serde_json::to_vec_pretty(cfg).unwrap();
    fs::write(path, &bytes).unwrap();
    bytes
}
#[test]
fn decode_legacy_is_read_only_until_verified_preparation() {
    let f = Fixture::new();
    let mut cfg = station();
    cfg.presets[0].url = format!("  {} \r\n", cfg.presets[0].url);
    cfg.presets[0].metadata_url = "\thttps://synthetic.invalid/meta \n".into();
    cfg.custom_eq_presets.push(EqPreset {
        name: "rOCK".into(),
        preamp: 2.,
        bands: [2.; 10],
    });
    cfg.presets[0].eq = "rOCK".into();
    let original = raw(&f.path, &cfg);
    let c = config::load_or_default(&f.path).unwrap();
    assert_eq!(fs::read(&f.path).unwrap(), original);
    assert!(f.backups().is_empty());
    assert_eq!(c.presets[0].eq, "Imported EQ 1");
    assert!(!c.presets[0].url.starts_with(' '));
    drop(f.prepare().unwrap());
    assert!(f.backups().iter().any(|p| fs::read(p).unwrap() == original));
    let document: serde_json::Value = serde_json::from_slice(&fs::read(&f.path).unwrap()).unwrap();
    assert!(document.get("stationBanks").is_some());
    assert!(document.get("presets").is_none());
    let count = f.backups().len();
    let canonical = fs::read(&f.path).unwrap();
    drop(f.prepare().unwrap());
    assert_eq!(fs::read(&f.path).unwrap(), canonical);
    assert_eq!(f.backups().len(), count);
}
#[test]
fn fresh_restart_and_lock_are_isolated() {
    let f = Fixture::new();
    let first = f.prepare().unwrap();
    assert_eq!(first.origin, "fresh");
    assert!(f.path.is_file());
    assert!(first.webview.starts_with(&f.app));
    assert!(!f.root.join("webview").exists());
    assert!(f.prepare().is_err());
    let cache = first.webview.clone();
    drop(first);
    let before = fs::read(&f.path).unwrap();
    let second = f.prepare().unwrap();
    assert_eq!(second.webview, cache);
    assert_eq!(fs::read(&f.path).unwrap(), before);
    assert!(f.backups().is_empty());
}
#[test]
fn migrate_previous_then_save_restart_and_detect_old_changes() {
    let f = Fixture::new();
    let original = raw(&f.legacy, &station());
    let oldcache = f.root.join("local-data/player/webview/EBWebView");
    fs::create_dir_all(&oldcache).unwrap();
    fs::write(oldcache.join("marker"), b"unchanged").unwrap();
    let mut p = f.prepare().unwrap();
    assert_eq!(p.origin, "previous-profile");
    assert_eq!(fs::read(&f.legacy).unwrap(), original);
    assert!(f.backups().iter().any(|p| fs::read(p).unwrap() == original));
    p.config.volume = 19;
    p.config.muted = true;
    p.config.window_x = 123;
    config::save(&f.path, &p.config).unwrap();
    drop(p);
    let before = fs::read(&f.path).unwrap();
    let p = f.prepare().unwrap();
    assert_eq!(p.config.volume, 19);
    assert!(p.config.muted);
    drop(p);
    assert_eq!(fs::read(&f.path).unwrap(), before);
    assert_eq!(fs::read(&f.legacy).unwrap(), original);
    assert_eq!(fs::read(oldcache.join("marker")).unwrap(), b"unchanged");
    let mut changed = station();
    changed.volume = 77;
    raw(&f.legacy, &changed);
    assert!(f.prepare().is_err());
    assert_eq!(fs::read(&f.path).unwrap(), before);
}
#[test]
fn ambiguous_configs_fail_before_any_backup_or_replacement() {
    let f = Fixture::new();
    let a = raw(&f.path, &station());
    let mut other = station();
    other.presets[0].url = "https://synthetic.invalid/different".into();
    let b = raw(&f.legacy, &other);
    assert!(f.prepare().is_err());
    assert_eq!(fs::read(&f.path).unwrap(), a);
    assert_eq!(fs::read(&f.legacy).unwrap(), b);
    assert!(f.backups().is_empty());
}
#[test]
fn equivalent_configs_acknowledge_legacy_without_rewriting_canonical_file() {
    let f = Fixture::new();
    let c = station();
    config::save(&f.path, &c).unwrap();
    let a = fs::read(&f.path).unwrap();
    let b = raw(&f.legacy, &c);
    let mut p = f.prepare().unwrap();
    assert_eq!(fs::read(&f.path).unwrap(), a);
    p.config.presets[0].name = "Changed only in active file".into();
    config::save(&f.path, &p.config).unwrap();
    drop(p);
    assert!(f.prepare().is_ok());
    assert_eq!(fs::read(&f.legacy).unwrap(), b);
}
#[test]
fn unknown_missing_ambiguous_and_nested_fields_never_rewrite() {
    let mut cases = Vec::new();
    let valid = serde_json::to_value(station()).unwrap();
    let mut c = valid.clone();
    let a = c.as_object_mut().unwrap().remove("presets").unwrap();
    c["stationbanks"] = a;
    cases.push(c);
    let mut c = valid.clone();
    c.as_object_mut().unwrap().remove("presets");
    cases.push(c);
    let mut c = valid.clone();
    c["stationBanks"] = serde_json::json!([]);
    cases.push(c);
    let mut c = valid.clone();
    c["presets"] = serde_json::Value::Null;
    cases.push(c);
    let mut c = valid.clone();
    c["unexpected"] = 1.into();
    cases.push(c);
    let mut c = valid.clone();
    c["presets"][0]["urll"] = "https://synthetic.invalid/hidden".into();
    cases.push(c);
    let mut c = config::stored_value(&station());
    c["stationBanks"][0]["slots"][0]["urll"] = "https://synthetic.invalid/hidden".into();
    cases.push(c);
    let mut c = config::stored_value(&station());
    c["stationBanks"][0]
        .as_object_mut()
        .unwrap()
        .remove("slots");
    cases.push(c);
    let mut c = valid.clone();
    c["customEqPresets"] =
        serde_json::json!([{"name":"Synthetic","preamp":0,"bands":vec![0;10],"gain":8}]);
    cases.push(c);
    for c in cases {
        let f = Fixture::new();
        let bytes = serde_json::to_vec(&c).unwrap();
        fs::write(&f.path, &bytes).unwrap();
        raw(&f.legacy, &station());
        assert!(config::load_or_default(&f.path).is_err());
        assert!(f.prepare().is_err());
        assert_eq!(fs::read(&f.path).unwrap(), bytes);
        assert!(f.backups().is_empty());
    }
}
#[test]
fn duplicate_json_fields_and_malformed_json_fail_closed() {
    let doc = serde_json::to_string(&station()).unwrap();
    let variants = [
        "{".to_owned(),
        doc.replacen("\"volume\":50", "\"volume\":50,\"volume\":51", 1),
        doc.replacen("\"eq\":\"Off\"", "\"eq\":\"Off\",\"eq\":\"Rock\"", 1),
    ];
    for bytes in variants {
        let f = Fixture::new();
        fs::write(&f.path, &bytes).unwrap();
        assert!(f.prepare().is_err());
        assert_eq!(fs::read_to_string(&f.path).unwrap(), bytes);
        assert!(f.backups().is_empty());
    }
}
#[test]
fn explicit_import_conflict_preserves_both_files_and_noop_import_is_read_only() {
    let f = Fixture::new();
    let mut c = station();
    let source = f.root.join("chosen.json");
    let original = raw(&source, &c);
    let p = profile::prepare(&f.path, None, Some(&source), &f.app).unwrap();
    drop(p);
    assert_eq!(fs::read(&source).unwrap(), original);
    let canonical = fs::read(&f.path).unwrap();
    config::import_explicit(&source, &f.path).unwrap();
    assert_eq!(fs::read(&f.path).unwrap(), canonical);
    c.volume = 99;
    raw(&source, &c);
    assert!(profile::prepare(&f.path, None, Some(&source), &f.app).is_err());
    assert!(config::import_explicit(&source, &f.path).is_err());
    assert_eq!(fs::read(&f.path).unwrap(), canonical);
}
#[test]
fn readonly_config_and_unwritable_appdata_fail_without_substitution() {
    let f = Fixture::new();
    let original = raw(&f.path, &station());
    let original_permissions = fs::metadata(&f.path).unwrap().permissions();
    let mut readonly = original_permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&f.path, readonly).unwrap();
    let rejected = f.prepare().is_err();
    fs::set_permissions(&f.path, original_permissions).unwrap();
    assert!(rejected);
    assert_eq!(fs::read(&f.path).unwrap(), original);
    assert!(f.backups().is_empty());
    let g = Fixture::new();
    fs::write(&g.app, b"not a directory").unwrap();
    assert!(g.prepare().is_err());
    assert!(!g.path.exists());
}
#[test]
fn cache_permissions_fail_before_config_migration() {
    let f = Fixture::new();
    let original = raw(&f.path, &station());
    let canonical = fs::canonicalize(&f.root).unwrap();
    let cache = profile::webview_path(&f.app, &canonical);
    fs::create_dir_all(cache.parent().unwrap()).unwrap();
    fs::write(&cache, b"not a directory").unwrap();
    assert!(f.prepare().is_err());
    assert_eq!(fs::read(&f.path).unwrap(), original);
    assert!(f.backups().is_empty());
}
#[test]
fn cache_identity_separates_profiles_and_does_not_rely_on_working_directory() {
    let f = Fixture::new();
    let a = profile::webview_path(&f.app, &f.root);
    let b = profile::webview_path(&f.app, &f.root.join("other"));
    assert_ne!(a, b);
    assert_eq!(a, profile::webview_path(&f.app, &f.root));
    assert!(a.starts_with(&f.app));
}
#[test]
fn lost_or_tampered_receipt_backup_cannot_hide_a_changed_legacy_file() {
    let f = Fixture::new();
    raw(&f.legacy, &station());
    let mut p = f.prepare().unwrap();
    p.config.volume = 19;
    config::save(&f.path, &p.config).unwrap();
    drop(p);
    let before = fs::read(&f.path).unwrap();
    for backup in f.backups() {
        fs::write(backup, b"tampered").unwrap();
    }
    assert!(f.prepare().is_err());
    assert_eq!(fs::read(&f.path).unwrap(), before);
}

#[test]
fn canonical_fractional_eq_and_formatting_do_not_trigger_repeated_migration() {
    let f = Fixture::new();
    let mut cfg = station();
    cfg.custom_eq_presets.push(EqPreset {
        name: "Fractional synthetic".into(),
        preamp: 1.1,
        bands: [-2.3, 0.1, 0.3, 0.7, 1.1, 2.3, 4.7, 6.1, 8.3, 10.1],
    });
    cfg.presets[0].eq = "Fractional synthetic".into();
    config::save(&f.path, &cfg).unwrap();
    let bytes = fs::read(&f.path).unwrap();
    for _ in 0..3 {
        let p = f.prepare().unwrap();
        assert_eq!(p.config.custom_eq_presets[0].preamp, 1.1);
        drop(p);
        assert_eq!(fs::read(&f.path).unwrap(), bytes);
        assert!(f.backups().is_empty());
    }
    let document: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let formatted = serde_json::to_vec(&document).unwrap();
    fs::write(&f.path, &formatted).unwrap();
    drop(f.prepare().unwrap());
    assert_eq!(fs::read(&f.path).unwrap(), formatted);
    assert!(f.backups().is_empty());
}

#[test]
fn real_pre_window_failures_reach_safe_dialog_then_exit_without_config_changes() {
    use crate::startup_failure::{finish_setup, Failure, Kind};
    fn verify(error: config::ConfigError, expected: Kind) {
        let failure = Failure::profile(error);
        assert_eq!(failure.0, expected);
        let calls = std::cell::RefCell::new(Vec::new());
        finish_setup(
            Err(failure),
            |message| {
                assert!(!message.to_string().contains("PRIVATE_SENTINEL"));
                calls.borrow_mut().push("dialog");
            },
            |code| {
                assert_eq!(code, 1);
                calls.borrow_mut().push("exit");
            },
        );
        assert_eq!(*calls.borrow(), ["dialog", "exit"]);
    }
    let f = Fixture::new();
    let a = raw(&f.path, &station());
    let mut c = station();
    c.volume = 19;
    let b = raw(&f.legacy, &c);
    verify(f.prepare().err().unwrap(), Kind::ConfigConflict);
    assert_eq!(fs::read(&f.path).unwrap(), a);
    assert_eq!(fs::read(&f.legacy).unwrap(), b);
    assert!(f.backups().is_empty());
    let f = Fixture::new();
    let mut value = serde_json::to_value(station()).unwrap();
    value["volume"] = "PRIVATE_SENTINEL".into();
    let bytes = serde_json::to_vec(&value).unwrap();
    fs::write(&f.path, &bytes).unwrap();
    verify(f.prepare().err().unwrap(), Kind::ConfigInvalid);
    assert_eq!(fs::read(&f.path).unwrap(), bytes);
    let f = Fixture::new();
    let bytes = raw(&f.path, &station());
    let original = fs::metadata(&f.path).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&f.path, readonly).unwrap();
    let error = f.prepare().err().unwrap();
    fs::set_permissions(&f.path, original).unwrap();
    verify(error, Kind::Storage);
    assert_eq!(fs::read(&f.path).unwrap(), bytes);
    let f = Fixture::new();
    let active = f.prepare().unwrap();
    verify(f.prepare().err().unwrap(), Kind::ProfileBusy);
    drop(active);
    let f = Fixture::new();
    fs::write(&f.app, b"not a directory").unwrap();
    verify(f.prepare().err().unwrap(), Kind::Storage);
    assert!(!f.path.exists());
}
