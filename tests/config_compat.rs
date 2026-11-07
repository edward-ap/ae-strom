use ae_strom::{
    config::{self, ConfigError},
    config_issue::FieldPath,
    model::EqPreset,
    startup_failure::Failure,
};
use serde_json::{json, Value};
fn legacy() -> Value {
    serde_json::to_value(config::default_config()).unwrap()
}
fn error(v: &Value) -> (ConfigError, String) {
    let e = config::decode_compatible(&v.to_string()).unwrap_err();
    let text = Failure::profile(config::decode_compatible(&v.to_string()).unwrap_err()).to_string();
    assert!(!text.to_ascii_uppercase().contains("PRIVATE_SENTINEL"));
    (e, text)
}
fn path(v: &Value, expected: &str) {
    let (e, text) = error(v);
    match e {
        ConfigError::Issue(issue) => assert_eq!(issue.path.to_string(), expected),
        _ => panic!("Expected a typed diagnostic"),
    };
    assert!(text.contains(expected));
}

#[test]
fn legacy_path_query_spaces_and_unicode_are_preserved_without_request_or_rewrite() {
    let mut v = legacy();
    let url = "https://example.invalid/старый путь?title=a b&token=PRIVATE_SENTINEL";
    v["presets"][0]["url"] = url.into();
    let cfg = config::decode_compatible(&v.to_string()).unwrap();
    assert_eq!(cfg.presets[0].url, url);
    let parsed = url::Url::parse(url).unwrap();
    assert!(parsed.as_str().contains("a%20b"));
    v["presets"][0]["url"] = "https://exa mple.invalid/private?token=PRIVATE_SENTINEL".into();
    path(&v, "presets[0].url");
    v["presets"][0]["url"] = "https://example.invalid/a\nb?token=PRIVATE_SENTINEL".into();
    path(&v, "presets[0].url");
}
#[test]
fn legacy_long_unicode_and_padded_eq_names_keep_curve_and_binding() {
    for name in ["Эквалайзер ".repeat(30), "  Legacy Room  ".into()] {
        let mut cfg = config::default_config();
        cfg.custom_eq_presets.push(EqPreset {
            name: name.clone(),
            preamp: 2.,
            bands: [1.; 10],
        });
        cfg.presets[0].eq = name.clone();
        let decoded = config::decode_compatible(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(decoded.custom_eq_presets[0].name, name);
        assert_eq!(decoded.presets[0].eq, name);
        assert_eq!(decoded.custom_eq_presets[0].bands, [1.; 10]);
        assert!(config::validate_custom_eq_name(&name).is_err());
    }
}
#[test]
fn explicit_empty_banks_preserve_later_slots_but_missing_arrays_remain_ambiguous() {
    let mut v = legacy();
    let station = v["presets"][0].clone();
    v.as_object_mut().unwrap().remove("presets");
    let mut s = station;
    s["key"] = "0".into();
    s["url"] = "https://synthetic.invalid/z".into();
    v["stationBanks"] = json!([{"bank":"a","slots":[]},{"bank":"Z","slots":[s]}]);
    let cfg = config::decode_compatible(&v.to_string()).unwrap();
    assert_eq!(cfg.presets.len(), 260);
    assert_eq!(cfg.presets[259].url, "https://synthetic.invalid/z");
    v["stationBanks"][0]
        .as_object_mut()
        .unwrap()
        .remove("slots");
    path(&v, "stationBanks[0].slots");
}
#[test]
fn unknown_eq_and_case_collisions_are_actionable_and_never_replaced_with_off() {
    let mut v = legacy();
    v["presets"][0]["eq"] = "PRIVATE_SENTINEL_missing_curve".into();
    path(&v, "presets[0].eq");
    assert!(error(&v).1.contains("Off was not substituted"));
    v = legacy();
    v["customEqPresets"] = json!([{"name":"PRIVATE_SENTINEL_Name","preamp":0,"bands":[0,0,0,0,0,0,0,0,0,0]},{"name":"private_sentinel_name","preamp":0,"bands":[0,0,0,0,0,0,0,0,0,0]}]);
    path(&v, "customEqPresets[1].name");
}
#[test]
fn value_failures_identify_each_known_field_without_disclosing_its_value() {
    let mut v = legacy();
    v["volume"] = 101.into();
    path(&v, "volume");
    v = legacy();
    v["presets"][3]["metadataType"] = "PRIVATE_SENTINEL".into();
    path(&v, "presets[3].metadataType");
    v = legacy();
    v["presets"][4]["metadataUrl"] = "file:///PRIVATE_SENTINEL".into();
    path(&v, "presets[4].metadataUrl");
    v = legacy();
    v["customEqPresets"] =
        json!([{"name":"PRIVATE_SENTINEL","preamp":16,"bands":[0,0,0,0,0,0,0,0,0,0]}]);
    path(&v, "customEqPresets[0].preamp");
    v["customEqPresets"][0]["preamp"] = 0.into();
    v["customEqPresets"][0]["bands"][7] = 16.into();
    path(&v, "customEqPresets[0].bands[7]");
}
#[test]
fn sparse_diagnostics_point_to_source_array_indices_not_flattened_padding() {
    let mut v = legacy();
    let mut station = v["presets"][0].clone();
    station["key"] = "0".into();
    station["eq"] = "PRIVATE_SENTINEL".into();
    v.as_object_mut().unwrap().remove("presets");
    v["stationBanks"] = json!([{"bank":"A","slots":[]},{"bank":"Z","slots":[station]}]);
    path(&v, "stationBanks[1].slots[0].eq");
    v["stationBanks"][1]["slots"][0]["eq"] = "Off".into();
    v["stationBanks"][1]["slots"][0]["key"] = "PRIVATE_SENTINEL".into();
    path(&v, "stationBanks[1].slots[0].key");
    v["stationBanks"][1]["bank"] = "PRIVATE_SENTINEL".into();
    path(&v, "stationBanks[1].bank");
}
#[test]
fn structural_failures_have_safe_locations_including_secret_unknown_keys() {
    let mut v = legacy();
    v["volume"] = "PRIVATE_SENTINEL".into();
    path(&v, "volume");
    v = legacy();
    v["presets"][2]["url"] = false.into();
    path(&v, "presets[2].url");
    v = legacy();
    v["customEqPresets"] = json!([{"name":"PRIVATE_SENTINEL","preamp":0,"bands":[0,0]}]);
    path(&v, "customEqPresets[0].bands");
    v = legacy();
    v["PRIVATE_SENTINEL"] = "secret".into();
    path(&v, "$");
}
#[test]
fn malformed_and_duplicate_json_locations_do_not_echo_input() {
    for raw in ["{\"PRIVATE_SENTINEL\":", "{\"volume\":1,\"volume\":2}"] {
        let e = config::decode_compatible(raw).unwrap_err();
        match &e {
            ConfigError::Issue(i) => assert!(matches!(i.path, FieldPath::Json { .. })),
            _ => panic!("Missing safe JSON location"),
        };
        let text = Failure::profile(e).to_string();
        assert!(!text.to_ascii_uppercase().contains("PRIVATE_SENTINEL"));
        assert!(text.contains("JSON line"));
    }
}
#[test]
fn supported_import_roundtrip_preserves_source_and_rejected_import_never_writes() {
    let root = std::env::temp_dir().join(format!(
        "ae-config-compat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    assert!(root
        .canonicalize()
        .unwrap()
        .starts_with(std::env::temp_dir().canonicalize().unwrap()));
    let source = root.join("source.json");
    let destination = root.join("destination.json");
    let mut v = legacy();
    let name = "Старый эквалайзер ".repeat(12);
    v["customEqPresets"] = json!([{"name":name,"preamp":1,"bands":[0,0,0,0,0,0,0,0,0,0]}]);
    v["presets"][0]["eq"] = name.clone().into();
    v["presets"][0]["url"] = "https://synthetic.invalid/a b?token=PRIVATE_SENTINEL".into();
    let bytes = serde_json::to_vec(&v).unwrap();
    std::fs::write(&source, &bytes).unwrap();
    config::import_explicit(&source, &destination).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), bytes);
    let loaded = config::load_or_default(&destination).unwrap();
    assert_eq!(loaded.presets[0].eq, name);
    assert_eq!(
        loaded.presets[0].url,
        v["presets"][0]["url"].as_str().unwrap()
    );
    let target = std::fs::read(&destination).unwrap();
    v["presets"][0]["eq"] = "PRIVATE_SENTINEL_missing".into();
    let rejected = serde_json::to_vec(&v).unwrap();
    std::fs::write(&source, &rejected).unwrap();
    assert!(config::import_explicit(&source, &destination).is_err());
    assert_eq!(std::fs::read(&source).unwrap(), rejected);
    assert_eq!(std::fs::read(&destination).unwrap(), target);
    assert!(!root.join("roundtrip-config.json").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn excess_slots_duplicates_url_limits_and_extreme_gains_do_not_lose_data() {
    let mut v = legacy();
    let station = v["presets"][0].clone();
    v["presets"] = Value::Array(vec![station.clone(); 261]);
    path(&v, "presets");
    v = legacy();
    let mut slot = station;
    slot["key"] = "1".into();
    v.as_object_mut().unwrap().remove("presets");
    v["stationBanks"] = json!([{"bank":"A","slots":[slot.clone(),slot]}]);
    path(&v, "stationBanks[0].slots[1].key");
    v = legacy();
    v["presets"][0]["metadataUrl"] =
        format!("https://synthetic.invalid/{}", "x".repeat(16384)).into();
    path(&v, "presets[0].metadataUrl");
    v = legacy();
    v["customEqPresets"] =
        json!([{"name":"PRIVATE_SENTINEL","preamp":1e100,"bands":[0,0,0,0,0,0,0,0,0,0]}]);
    path(&v, "customEqPresets[0].preamp");
}
