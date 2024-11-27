use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

pub const FOLDER_SCHEMA: usize = 0;

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

pub fn fatal_error_close_windows(handle: &AppHandle, msg: &str) -> ! {
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
pub fn check_full_disk(home_dir: &Path) -> bool {
    // macOS has no api for telling whether we have full disk access directly
    // there are some directories we can try reading to tell whether we have access or not
    // based off of https://github.com/MacPaw/PermissionsKit/blob/master/PermissionsKit/Private/FullDiskAccess/MPFullDiskAccessAuthorizer.m

    let test_paths: [PathBuf; 4] = [
        home_dir.join("Library/Safari/CloudTabs.db"),
        home_dir.join("Library/Safari/Bookmarks.plist"),
        home_dir.join("Library/Application Support/com.apple.TCC/TCC.db"),
        PathBuf::from("/Library/Preferences/com.apple.TimeMachine.plist"),
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

pub fn hash_file(file: &Path) -> Result<String> {
    let mut file = std::fs::File::open(file).map_err(|e| {
        anyhow!(
            "Failed to open file {} to calculate hash, error: {e}",
            path_to_str(file)
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Gets a filename from a path which is expected to be a file.
pub fn get_filename(path: &Path) -> Result<String> {
    Ok(path
        .file_name()
        .ok_or(anyhow!(
            "File {} does not have a filename",
            path.to_string_lossy()
        ))?
        .to_string_lossy()
        .to_string())
}

// i know windows paths are funky which is why i have this
pub fn path_to_str(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn get_unix_timestamp() -> Result<u64> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())? as u64)
}

// pub fn is_valid_ulid(s: &str) -> bool {
//     s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric())
// }

pub fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Extracts the first two characters from a ulid's hash part
pub fn get_ulid_chars(ulid: &str) -> String {
    ulid[10..=11].into()
}

/// A unique directory within a directory.
pub struct UniqueDir {
    pub name: String,
    pub path: PathBuf,
}

impl UniqueDir {
    pub fn new<P>(dir: P, name: &str) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        fs::create_dir_all(dir.as_ref())
            .context("Failed to create missing parent directory for UniqueDir")?;

        let create = |p: &Path| match fs::create_dir(p) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(e),
        };

        let mut path = dir.as_ref().join(name);

        if create(&path)? {
            Ok(Self {
                name: name.to_string(),
                path,
            })
        } else {
            let mut count = 1usize;
            loop {
                let curr_name = format!("{name}-{count}");
                path = dir.as_ref().join(&curr_name);

                if create(&path)? {
                    break Ok(Self {
                        name: curr_name,
                        path,
                    });
                }

                count += 1;
            }
        }
    }
}

/// A unique file within a directory.
pub struct UniqueFile {
    pub filename: String,
    pub path: PathBuf,
    pub file: fs::File,
}

impl UniqueFile {
    pub fn new<P>(dir: P, filename: &str) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut path = dir.as_ref().join(filename);

        if !fs::exists(&path)? {
            Ok(Self {
                filename: filename.to_string(),
                file: fs::File::create_new(&path)?,
                path,
            })
        } else {
            let (base, ext) = extract_file_extension(filename);

            let mut count = 1usize;
            loop {
                let curr_filename = format!("{base}-{count}{}", ext.as_deref().unwrap_or(""));
                path = dir.as_ref().join(&curr_filename);

                if !path.exists() {
                    break Ok(Self {
                        filename: curr_filename,
                        file: fs::File::create_new(&path)?,
                        path,
                    });
                }

                count += 1;
            }
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn file(&mut self) -> &mut fs::File {
        &mut self.file
    }
}

pub struct TempDir {
    path: std::path::PathBuf,
    persist: bool,
    save_on_panic: bool,
    panic_if_err: bool,
}

impl TempDir {
    /// Gets a new temporary directory.
    pub fn new<P>(parent_dir: P, prefix: &str) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let UniqueDir { name: _, path } = UniqueDir::new(parent_dir, prefix)?;

        log::info!("Created a temporary directory: {}", path_to_str(&path));

        Ok(Self {
            path,
            persist: false,
            save_on_panic: false,
            panic_if_err: true,
        })
    }

    pub fn persist(&mut self) {
        self.persist = true
    }

    /// Does not delete this temporary directory when a panic occurs.
    pub fn save_on_panic(&mut self) {
        self.save_on_panic = true
    }

    /// Panics if an error is encountered when deleting this temporary directory when `Drop`ping.
    pub fn panic_if_err(&mut self) {
        self.panic_if_err = true
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !((std::thread::panicking() && self.save_on_panic) || self.persist) {
            match std::fs::remove_dir_all(&self.path) {
                Ok(()) => log::info!("Removed temporary directory at {}", path_to_str(&self.path)),
                Err(e) => {
                    if self.panic_if_err {
                        panic!("Error while deleting temporary directory: {}", e)
                    } else {
                        log::error!("Error while deleting temporary directory: {}", e)
                    }
                }
            }
        }
    }
}
