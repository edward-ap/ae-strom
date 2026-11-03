//! Quiet, explicit live-mode launcher. It never starts playback or imports data.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::path::{Path, PathBuf};

fn launch_paths(directory: &Path) -> Result<(PathBuf, PathBuf), String> {
    let (gui, root) = if directory.join("target/release/ae-strom.exe").is_file() {
        (
            directory.join("target/release/ae-strom.exe"),
            directory.to_path_buf(),
        )
    } else if directory.file_name().is_some_and(|s| s == "release")
        && directory
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|s| s == "target")
    {
        (
            directory.join("ae-strom.exe"),
            directory
                .parent()
                .and_then(Path::parent)
                .ok_or("Missing project directory")?
                .to_path_buf(),
        )
    } else {
        (directory.join("ae-strom.exe"), directory.to_path_buf())
    };
    if !gui.is_file() {
        return Err(
            "The main ae-strom.exe was not found beside this launcher or in target/release.".into(),
        );
    }
    Ok((gui, root.join("local-data/player")))
}

fn run() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|_| "Cannot locate the launcher")?;
    let (gui, profile) = launch_paths(exe.parent().ok_or("Missing launcher directory")?)?;
    std::process::Command::new(gui)
        .arg("--enable-live-audio").arg("--data-dir").arg(profile)
        .spawn().map_err(|_| "Cannot start the player. Rebuild with the live-audio feature and keep the main EXE in its expected location.")?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
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
            let text: Vec<u16> = error.encode_utf16().chain(Some(0)).collect();
            let caption: Vec<u16> = "Æ Strøm".encode_utf16().chain(Some(0)).collect();
            // Standard OS error dialog; no shell, script host or console window.
            unsafe {
                MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
            }
        }
        #[cfg(not(windows))]
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "ae-strom-launcher-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        root
    }
    #[test]
    fn root_launcher_uses_explicit_project_profile() {
        let root = fixture();
        let gui = root.join("target/release/ae-strom.exe");
        std::fs::create_dir_all(gui.parent().unwrap()).unwrap();
        std::fs::write(&gui, []).unwrap();
        assert_eq!(
            launch_paths(&root).unwrap(),
            (gui, root.join("local-data/player"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn release_launcher_resolves_same_project_profile() {
        let root = fixture();
        let dir = root.join("target/release");
        std::fs::create_dir_all(&dir).unwrap();
        let gui = dir.join("ae-strom.exe");
        std::fs::write(&gui, []).unwrap();
        assert_eq!(
            launch_paths(&dir).unwrap(),
            (gui, root.join("local-data/player"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn portable_bundle_keeps_profile_beside_its_executables() {
        let root = fixture();
        let gui = root.join("ae-strom.exe");
        std::fs::write(&gui, []).unwrap();
        assert_eq!(
            launch_paths(&root).unwrap(),
            (gui, root.join("local-data/player"))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn missing_main_executable_is_rejected() {
        let root = fixture();
        assert!(launch_paths(&root).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
