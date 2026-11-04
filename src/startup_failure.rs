//! Sanitized startup notifications shared by pre-runtime and deferred setup failures.
//! Raw config/serde/OS error strings never reach the GUI or stderr.
use crate::config::ConfigError;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Arguments,
    Paths,
    AudioBuild,
    AppData,
    ConfigConflict,
    ImportConflict,
    ConfigInvalid,
    ConfigValues,
    Storage,
    ProfileBusy,
    Migration,
    Controller,
    WebViewRuntime,
    WebView,
    Runtime,
}
#[derive(Clone, Copy, Debug)]
pub struct Failure(pub Kind);
impl Failure {
    pub fn profile(error: ConfigError) -> Self {
        match error {
            ConfigError::Json(_) => Self(Kind::ConfigInvalid),
            ConfigError::Io(e) => {
                let text = e.to_string();
                let kind = if text
                    .starts_with("Adjacent config.json and local-data/player/config.json differ")
                {
                    Kind::ConfigConflict
                } else if text.starts_with("Import conflicts with existing config") {
                    Kind::ImportConflict
                } else if text.starts_with("This profile is already open") {
                    Kind::ProfileBusy
                } else if text.contains("receipt")
                    || text.contains("backup")
                    || text.contains("changed during")
                    || text.contains("roundtrip")
                {
                    Kind::Migration
                } else if text.starts_with("Unrecognized or incomplete configuration")
                    || text.contains("must be UTF-8")
                {
                    Kind::ConfigInvalid
                } else if e.kind() == std::io::ErrorKind::InvalidData
                    && !text.starts_with("Config is read-only")
                {
                    Kind::ConfigValues
                } else {
                    Kind::Storage
                };
                Self(kind)
            }
        }
    }
    pub fn code(self) -> &'static str {
        match self.0 {
            Kind::Arguments => "ARGUMENTS",
            Kind::Paths => "PATHS",
            Kind::AudioBuild => "AUDIO-BUILD",
            Kind::AppData => "APPDATA",
            Kind::ConfigConflict => "CONFIG-CONFLICT",
            Kind::ImportConflict => "IMPORT-CONFLICT",
            Kind::ConfigInvalid => "CONFIG-STRUCTURE",
            Kind::ConfigValues => "CONFIG-VALUES",
            Kind::Storage => "STORAGE",
            Kind::ProfileBusy => "PROFILE-IN-USE",
            Kind::Migration => "MIGRATION",
            Kind::Controller => "PLAYER-INIT",
            Kind::WebViewRuntime => "WEBVIEW2-RUNTIME",
            Kind::WebView => "WEBVIEW",
            Kind::Runtime => "RUNTIME",
        }
    }
    pub fn guidance(self) -> &'static str {
        match self.0 {
  Kind::Arguments=>"The launch arguments are invalid. Check --fake, --data-dir <absolute directory>, or --import-config <file>.",
  Kind::Paths=>"The application could not determine its executable location. Start it from an accessible local folder.",
  Kind::AudioBuild=>"This executable was built without native audio. Use a full build or explicit --fake mode.",
  Kind::AppData=>"The Windows user AppData location is unavailable. Check that your local user profile is accessible and writable.",
  Kind::ConfigConflict=>"config.json beside the EXE and local-data/player/config.json contain different settings. Both files were preserved. Choose which profile to keep, or test a copy with --data-dir. No empty profile was substituted.",
  Kind::ImportConflict=>"The import would replace a different existing config.json. Both files were preserved. Import into a fresh --data-dir instead.",
  Kind::ConfigInvalid=>"A configuration or migration record is malformed, incomplete, or has unknown/duplicate fields. Check the JSON and use exactly one station-storage field: presets or stationBanks. No empty profile was substituted.",
  Kind::ConfigValues=>"The configuration contains invalid URLs, EQ references/values, metadata settings, or import parameters. Correct a copy or restore a verified backup. No empty profile was substituted.",
  Kind::Storage=>"The application cannot read or write its configuration or AppData storage. Check folder/file permissions, read-only attributes and free space. Keep existing configs and backups; no alternate profile was selected.",
  Kind::ProfileBusy=>"This profile is already open. Close its other player window before starting another copy. Existing settings were preserved.",
  Kind::Migration=>"The previous profile, backup or migration record could not be verified, or changed during preparation. Keep both configs and backups. Select an explicit copied profile with --data-dir; no automatic choice was made.",
  Kind::Controller=>"The native player could not initialize. Check the selected profile and available audio output. Keep your config and backups; no alternate profile was selected.",
  Kind::WebViewRuntime=>"Microsoft Edge WebView2 Runtime is unavailable for this Windows user. Install or repair the Evergreen Runtime from the official Microsoft page:\n\nhttps://developer.microsoft.com/en-us/microsoft-edge/webview2/\n\nThe player will not download or install it automatically. Close this message, install the Runtime yourself, then start the player again.",
  Kind::WebView=>"The player window could not initialize. Check the Windows WebView2 Runtime and writable AppData storage. Keep your config and backups.",
  Kind::Runtime=>"The desktop runtime could not initialize. Check Windows WebView2 availability and user storage access. Keep your config and backups.",
 }
    }
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Unable to start Æ Strøm.\n\n{}\n\nError code: {}",
            self.guidance(),
            self.code()
        )
    }
}
impl std::error::Error for Failure {}

/// Tauri runs setup inside Ready and panics on an escaped Err. Report first,
/// request a controlled exit, then let the caller return Ok to the runtime.
pub fn finish_setup(
    result: Result<(), Failure>,
    present: impl FnOnce(&Failure),
    exit: impl FnOnce(i32),
) {
    if let Err(error) = result {
        present(&error);
        exit(1);
    }
}

pub fn present(error: &Failure) {
    eprintln!("{error}");
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(
                window: *mut std::ffi::c_void,
                text: *const u16,
                caption: *const u16,
                flags: u32,
            ) -> i32;
        }
        let text: Vec<u16> = error.to_string().encode_utf16().chain(Some(0)).collect();
        let caption: Vec<u16> = "Æ Strøm — startup error"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        const MB_ICONERROR: u32 = 0x10;
        const MB_SETFOREGROUND: u32 = 0x10000;
        const MB_TOPMOST: u32 = 0x40000;
        let shown = unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                caption.as_ptr(),
                MB_ICONERROR | MB_SETFOREGROUND | MB_TOPMOST,
            )
        };
        if shown == 0 {
            eprintln!(
                "Startup error dialog could not be displayed; error code: {}",
                error.code()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn setup_reports_once_before_exit_for_every_failure_category() {
        for kind in [
            Kind::Arguments,
            Kind::Paths,
            Kind::AudioBuild,
            Kind::AppData,
            Kind::ConfigConflict,
            Kind::ImportConflict,
            Kind::ConfigInvalid,
            Kind::ConfigValues,
            Kind::Storage,
            Kind::ProfileBusy,
            Kind::Migration,
            Kind::Controller,
            Kind::WebViewRuntime,
            Kind::WebView,
            Kind::Runtime,
        ] {
            let calls = std::cell::RefCell::new(Vec::new());
            finish_setup(
                Err(Failure(kind)),
                |e| {
                    assert!(!e.guidance().is_empty());
                    calls.borrow_mut().push("dialog");
                },
                |code| {
                    assert_eq!(code, 1);
                    calls.borrow_mut().push("exit");
                },
            );
            assert_eq!(*calls.borrow(), ["dialog", "exit"]);
        }
    }
    #[test]
    fn successful_setup_does_not_notify_or_exit() {
        finish_setup(
            Ok(()),
            |_| panic!("unexpected dialog"),
            |_| panic!("unexpected exit"),
        );
    }
    #[test]
    fn raw_json_values_paths_and_io_messages_never_escape() {
        const PRIVATE: &str = "PRIVATE_SENTINEL_credential_and_station";
        let error = serde_json::from_str::<u8>(&format!("\"{PRIVATE}\"")).unwrap_err();
        assert!(error.to_string().contains(PRIVATE));
        let failures = [
            Failure::profile(ConfigError::Json(error)),
            Failure::profile(
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("C:/private/{PRIVATE}/config.json"),
                )
                .into(),
            ),
        ];
        for failure in failures {
            assert!(!failure.to_string().contains(PRIVATE));
            assert!(std::error::Error::source(&failure).is_none());
        }
    }
    #[test]
    fn profile_error_categories_preserve_actionable_safe_guidance() {
        for (message, kind) in [
            (
                "Adjacent config.json and local-data/player/config.json differ. PRIVATE",
                Kind::ConfigConflict,
            ),
            (
                "Import conflicts with existing config.json; PRIVATE",
                Kind::ImportConflict,
            ),
            ("This profile is already open; PRIVATE", Kind::ProfileBusy),
            (
                "Unrecognized or incomplete configuration structure; PRIVATE",
                Kind::ConfigInvalid,
            ),
            ("Config is read-only; PRIVATE", Kind::Storage),
            ("Config changed during backup; PRIVATE", Kind::Migration),
            (
                "Invalid station or metadata URL PRIVATE",
                Kind::ConfigValues,
            ),
        ] {
            let failure = Failure::profile(
                std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
            );
            assert_eq!(failure.0, kind);
            assert!(!failure.to_string().contains("PRIVATE"));
        }
    }
}
