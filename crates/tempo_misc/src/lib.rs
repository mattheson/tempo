pub mod types;
pub use crate::types::*;

use sha2::Digest;
use std::io::Read;
use tauri::Manager;

/// Opens blocking error dialog and closes afterwards.
pub fn fatal_error(msg: &str) -> ! {
    eprintln!("fatal error: {msg}");
    let _ = native_dialog::MessageDialog::new()
        .set_type(native_dialog::MessageType::Error)
        .set_title("Tempo: Fatal error")
        .set_text(msg)
        .show_alert();
    std::process::exit(1)
}

pub fn fatal_error_close_windows(handle: &tauri::AppHandle, msg: &str) -> ! {
    for (_, window) in handle.webview_windows() {
        let _ = window.close();
    }
    fatal_error(msg)
}

#[cfg(target_os = "macos")]
pub fn open_full_disk() {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let url_str = NSString::alloc(nil)
            .init_str("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles");
        let url: id = msg_send![class!(NSURL), URLWithString:url_str];
        let _: () = msg_send![workspace, openURL:url];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn open_full_disk() {
    log::warn!("called open_full_disk() on non-macOS");
}

/// Returns true if we have full disk access
#[cfg(target_os = "macos")]
pub fn check_full_disk(home_dir: &std::path::Path) -> bool {
    // macOS has no api for telling whether we have full disk access directly
    // there are some directories we can try reading to tell whether we have access or not
    // based off of https://github.com/MacPaw/PermissionsKit/blob/master/PermissionsKit/Private/FullDiskAccess/MPFullDiskAccessAuthorizer.m

    let test_paths: [std::path::PathBuf; 4] = [
        home_dir.join("Library/Safari/CloudTabs.db"),
        home_dir.join("Library/Safari/Bookmarks.plist"),
        home_dir.join("Library/Application Support/com.apple.TCC/TCC.db"),
        std::path::PathBuf::from("/Library/Preferences/com.apple.TimeMachine.plist"),
    ];

    for p in test_paths {
        if std::fs::File::open(p).is_ok() {
            return true;
        }
    }

    false
}

#[cfg(not(target_os = "macos"))]
pub fn check_full_disk() -> Result<bool> {
    Ok(true)
}

pub fn remove_file_extension(filename: &str) -> String {
    match filename.rfind('.') {
        Some(index) => filename[..index].to_string(),
        None => filename.to_string(),
    }
}

/// Returns `(filename without extension, extension)`
pub fn extract_file_extension(filename: &str) -> (String, Option<String>) {
    match filename.rfind('.') {
        Some(index) => (
            filename[..index].to_string(),
            Some(filename[index + 1..].to_string()),
        ),
        None => (filename.into(), None),
    }
}

pub fn hash_file(file: &std::path::Path) -> std::io::Result<Sha256Hash> {
    let mut file = std::fs::File::open(file)?;

    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(Sha256Hash(format!("{:x}", hasher.finalize())))
}

/// Reads data and copies it back, computes SHA256.
pub fn hash_and_copy(
    mut input: impl std::io::Read,
    mut output: impl std::io::Write,
) -> std::io::Result<Sha256Hash> {
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        output.write_all(&buffer)?;
    }

    Ok(Sha256Hash(format!("{:x}", hasher.finalize())))
}

/// Gets a filename from a path. Does not check whether the path is a directory or a file.
pub fn get_filename(path: &std::path::Path) -> Option<String> {
    Some(path.file_name()?.to_string_lossy().to_string())
}

// i know windows paths are funky which is why i have this, probably will be unused though
pub fn path_to_str(path: &std::path::Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn get_unix_timestamp() -> Result<u64, std::time::SystemTimeError> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())? as u64)
}

pub fn tauri_test() -> (
    tauri::App<tauri::test::MockRuntime>,
    tauri::AppHandle<tauri::test::MockRuntime>,
) {
    let app = tauri::test::mock_app();
    let handle = app.handle().clone();
    (app, handle)
}
