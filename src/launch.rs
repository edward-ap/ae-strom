//! Portable adjacent config, explicit test profiles, and read-only previous-profile discovery.
use crate::controller::PlaybackMode;
use std::path::{Path, PathBuf};

pub struct Options {
    pub data: PathBuf,
    pub legacy: Option<PathBuf>,
    pub mode: PlaybackMode,
    pub smoke: bool,
    pub startup_probe: bool,
    pub import: Option<PathBuf>,
    data_explicit: bool,
}

impl Options {
    /// macOS apps must never write inside their signed/read-only .app bundle.
    /// Explicit profiles retain exactly the same cross-platform behavior.
    pub fn use_app_data_profile(&mut self, app_local: &Path) -> Result<(), String> {
        if !app_local.is_absolute() {
            return Err("Application data directory must be absolute".into());
        }
        if self.startup_probe && !self.data_explicit {
            return Err("Startup smoke requires an explicit isolated --data-dir".into());
        }
        if !self.data_explicit {
            self.data = app_local.join(if self.mode == PlaybackMode::Live {
                "player"
            } else {
                "preview"
            });
            self.legacy = None;
        }
        Ok(())
    }
}

pub fn portable_root(executable: &Path) -> Result<PathBuf, String> {
    let directory = executable.parent().ok_or("Missing executable directory")?;
    // Cargo's development build uses the same profile as the canonical root EXE.
    // A plain portable distribution needs no source files or this layout.
    if directory.file_name().is_some_and(|s| s == "release")
        && directory
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|s| s == "target")
    {
        if let Some(root) = directory.parent().and_then(Path::parent) {
            if root.join("Cargo.toml").is_file() && root.join("src/main.rs").is_file() {
                return Ok(root.to_path_buf());
            }
        }
    }
    Ok(directory.to_path_buf())
}

pub fn parse(args: impl Iterator<Item = String>, executable: &Path) -> Result<Options, String> {
    let (mut data, mut mode, mut smoke, mut startup_probe, mut import) =
        (None, PlaybackMode::Live, false, false, None);
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data = Some(PathBuf::from(
                    args.next()
                        .ok_or("--data-dir requires an absolute directory")?,
                ))
            }
            "--fake" => mode = PlaybackMode::Fake,
            "--enable-live-audio" => mode = PlaybackMode::Live,
            "--smoke" => smoke = true,
            "--startup-smoke" => startup_probe = true,
            "--import-config" => {
                import = Some(PathBuf::from(
                    args.next()
                        .ok_or("--import-config requires a source file")?,
                ))
            }
            _ => return Err(format!("Unknown argument: {arg}")),
        }
    }
    if smoke && mode != PlaybackMode::Fake {
        return Err("Smoke requires explicit --fake; it never enables audio".into());
    }
    if smoke && startup_probe {
        return Err("Choose one smoke mode".into());
    }
    let data_explicit = data.is_some();
    let root = portable_root(executable)?;
    let legacy = if data.is_none() && mode == PlaybackMode::Live {
        Some(root.join("local-data/player/config.json"))
    } else {
        None
    };
    let data = data.unwrap_or_else(|| {
        if mode == PlaybackMode::Live {
            root.clone()
        } else {
            root.join("local-data/preview")
        }
    });
    if !data.is_absolute() {
        return Err("--data-dir must be absolute".into());
    }
    if startup_probe
        && std::fs::read_to_string(data.join("startup-probe.fixture"))
            .ok()
            .as_deref()
            != Some("ae-strom synthetic startup fixture\n")
    {
        return Err("Startup smoke requires a marked isolated synthetic fixture profile".into());
    }
    Ok(Options {
        data,
        legacy,
        mode,
        smoke,
        startup_probe,
        import,
        data_explicit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> PathBuf {
        // SystemTime can repeat across parallel tests on macOS.
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "ae-strom-routing-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
    fn opts(root: &Path, args: &[&str]) -> Result<Options, String> {
        parse(
            args.iter().map(|s| s.to_string()),
            &root.join("ae-strom.exe"),
        )
    }
    #[test]
    fn double_click_defaults_live_to_adjacent_player_not_preview() {
        let r = root();
        let o = opts(&r, &[]).unwrap();
        assert_eq!(o.mode, PlaybackMode::Live);
        assert_eq!(o.data, r);
        assert_eq!(o.legacy, Some(r.join("local-data/player/config.json")));
        assert!(!o.smoke && !o.startup_probe);
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn only_explicit_fake_uses_separate_preview() {
        let r = root();
        let o = opts(&r, &["--fake"]).unwrap();
        assert_eq!(o.mode, PlaybackMode::Fake);
        assert_eq!(o.data, r.join("local-data/preview"));
        assert!(opts(&r, &["--smoke"]).is_err());
        assert!(opts(&r, &["--fake", "--smoke"]).unwrap().smoke);
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn absolute_override_and_option_validation() {
        let r = root();
        let path = r.join("explicit profile");
        let o = opts(&r, &["--data-dir", path.to_str().unwrap()]).unwrap();
        assert_eq!(o.data, path);
        assert!(o.legacy.is_none());
        assert_eq!(o.mode, PlaybackMode::Live);
        for args in [
            &["--data-dir"][..],
            &["--data-dir", "relative"],
            &["--unknown"],
            &["--import-config"],
        ] {
            assert!(opts(&r, args).is_err());
        }
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn cargo_build_and_root_share_profile_without_cwd_dependency() {
        let r = root();
        std::fs::create_dir_all(r.join("src")).unwrap();
        std::fs::write(r.join("Cargo.toml"), []).unwrap();
        std::fs::write(r.join("src/main.rs"), []).unwrap();
        let build = r.join("target/release/ae-strom.exe");
        assert_eq!(portable_root(&build).unwrap(), r);
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn ordinary_portable_directory_needs_no_project_markers() {
        let r = root();
        let bundle = r.join("target/release");
        let o = parse(std::iter::empty(), &bundle.join("ae-strom.exe")).unwrap();
        assert_eq!(o.data, bundle);
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn startup_probe_does_not_change_default_mode_or_data() {
        let r = root();
        assert!(opts(&r, &["--startup-smoke"]).is_err());
        std::fs::create_dir_all(r.join("local-data/player")).unwrap();
        std::fs::write(
            r.join("startup-probe.fixture"),
            "ae-strom synthetic startup fixture\n",
        )
        .unwrap();
        let o = opts(&r, &["--startup-smoke"]).unwrap();
        assert_eq!(o.mode, PlaybackMode::Live);
        assert_eq!(o.data, r);
        assert_eq!(o.legacy, Some(r.join("local-data/player/config.json")));
        assert!(o.startup_probe);
        assert!(opts(&r, &["--fake", "--smoke", "--startup-smoke"]).is_err());
        std::fs::remove_dir_all(r).unwrap();
    }
    #[test]
    fn app_data_profile_never_writes_into_bundle_or_imports_legacy() {
        let r = root();
        let bundle = r.join("Æ Strøm.app/Contents/MacOS/ae-strom");
        let app_local = r.join("Application Support/local.edwardap.ae-strom");
        for (args, folder) in [(vec![], "player"), (vec!["--fake".to_owned()], "preview")] {
            let mut o = parse(args.into_iter(), &bundle).unwrap();
            o.use_app_data_profile(&app_local).unwrap();
            assert_eq!(o.data, app_local.join(folder));
            assert!(o.legacy.is_none());
            assert!(!o.data.starts_with(r.join("Æ Strøm.app")));
        }
        let explicit = r.join("explicit profile");
        let mut o = parse(
            [
                "--data-dir".to_owned(),
                explicit.to_string_lossy().into_owned(),
            ]
            .into_iter(),
            &bundle,
        )
        .unwrap();
        o.use_app_data_profile(&app_local).unwrap();
        assert_eq!(o.data, explicit);
        assert!(o.legacy.is_none());
        assert!(o.use_app_data_profile(Path::new("relative")).is_err());
        let mut implicit = opts(&r, &[]).unwrap();
        implicit.startup_probe = true;
        assert!(implicit.use_app_data_profile(&app_local).is_err());
        std::fs::remove_dir_all(r).unwrap();
    }
}
