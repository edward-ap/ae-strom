//! Detect the Windows WebView2 prerequisite before Tauri, profiles or playback initialize.
use crate::startup_failure::Failure;
#[cfg(any(windows, test))]
use crate::startup_failure::Kind;

/// Query only; never opens a window, downloads an installer or changes system state.
pub fn ensure_available() -> Result<(), Failure> {
    #[cfg(windows)]
    {
        validate_probe(tauri::webview_version().map_err(|_| ()))
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}

#[cfg(any(windows, test))]
fn validate_probe(probe: Result<String, ()>) -> Result<(), Failure> {
    let valid = probe.ok().is_some_and(|value| {
        let version = value.split_whitespace().next().unwrap_or_default();
        let parts: Option<Vec<u32>> = version.split('.').map(|part| part.parse().ok()).collect();
        parts.is_some_and(|parts| parts.len() == 4 && parts[0] > 0)
    });
    if valid {
        Ok(())
    } else {
        Err(Failure(Kind::WebViewRuntime))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_versions_and_preview_suffixes_are_accepted() {
        for version in ["149.0.7827.55", " 149.0.7827.55 ", "149.0.7827.55 beta"] {
            assert!(validate_probe(Ok(version.into())).is_ok());
        }
    }

    #[test]
    fn absent_failed_zero_or_malformed_probes_have_one_sanitized_error() {
        for probe in [
            Err(()),
            Ok(String::new()),
            Ok("0.0.0.0".into()),
            Ok("unknown".into()),
        ] {
            let failure = validate_probe(probe).unwrap_err();
            assert_eq!(failure.0, Kind::WebViewRuntime);
            assert_eq!(failure.code(), "WEBVIEW2-RUNTIME");
        }
    }

    #[test]
    fn runtime_guidance_identifies_manual_official_download_without_an_install_action() {
        let failure = validate_probe(Err(())).unwrap_err();
        let message = failure.to_string();
        assert!(message.contains("Microsoft Edge WebView2 Runtime"));
        assert!(message.contains("https://developer.microsoft.com/en-us/microsoft-edge/webview2/"));
        assert!(message.contains("will not download or install"));
        let calls = std::cell::RefCell::new(Vec::new());
        crate::startup_failure::finish_setup(
            Err(failure),
            |_| calls.borrow_mut().push("native-notification"),
            |code| {
                assert_eq!(code, 1);
                calls.borrow_mut().push("exit");
            },
        );
        assert_eq!(*calls.borrow(), ["native-notification", "exit"]);
    }
}
