use std::{
    io::Read,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, TempoError>;

#[derive(Debug, Error)]
pub enum TempoError {
    #[error("could not load tempo, no config found")]
    FailedToLoad,
    #[error("{0}")]
    Doc(String),
    #[error("the username {0} is already in use")]
    TakenUsername(String),
    #[error("{0}")]
    InvalidFolder(String),

    #[error("{0}")]
    Path(String),
    #[error("{0}")]
    File(String),
    #[error("{0}")]
    Folder(String),
    #[error("{0}")]
    Note(String),
    #[error("{0}")]
    Channel(String),
    #[error("{0}")]
    Config(String),
    #[error("{0}")]
    Db(String),

    #[error("{0}")]
    Daw(String),
    #[error("{0}")]
    Ableton(String),
    #[error("{0}")]
    Plugin(String),

    #[error("{0}")]
    MacOs(String),

    #[error("{0}")]
    Project(String),
    #[error("{0}")]
    Audio(String),

    // other errors
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error(transparent)]
    UlidEncode(#[from] ulid::EncodeError),
    #[error(transparent)]
    FromHex(#[from] hex::FromHexError),
    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("{0}")]
    ParseId(String),
    #[error(transparent)]
    Xml(#[from] quick_xml::Error),
    #[error(transparent)]
    XmlEncoding(#[from] quick_xml::encoding::EncodingError),
    #[error(transparent)]
    XmlEscape(#[from] quick_xml::escape::EscapeError),
    #[error(transparent)]
    Time(#[from] std::time::SystemTimeError),
    #[error(transparent)]
    Automerge(#[from] automerge::AutomergeError),
    #[error(transparent)]
    Reconcile(#[from] autosurgeon::ReconcileError),
    #[error(transparent)]
    Hydrate(#[from] autosurgeon::HydrateError),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("{0}")]
    Tauri(String),
}

impl From<TempoError> for String {
    fn from(value: TempoError) -> Self {
        value.to_string()
    }
}

impl From<tauri::Error> for TempoError {
    fn from(value: tauri::Error) -> Self {
        TempoError::Tauri(value.to_string())
    }
}

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

/*

impl TryFrom<Plugin> for PluginRow {
    type Error = TempoError;

    fn try_from(value: Plugin) -> Result<Self> {
        let name = if let Some(name) = value.name {
            name
        } else {
            return Err(TempoError::Db(format!(
                "Cannot convert Plugin into PluginRow, missing name for plugin: {:#?}",
                value.info
            )));
        };
        let vendor = if let Some(vendor) = value.vendor {
            vendor
        } else {
            return Err(TempoError::Db(format!(
                "Cannot convert Plugin into PluginRow, missing vendor for plugin: {:#?}",
                value.info
            )));
        };
        let (plugin_type, id) = match value.info {
            PluginInfo::Au {
                plugin_type,
                au_subtype,
                au_manufacturer,
            } => {
                let bytes: [u8; 12] = transmute!([au_type, au_subtype, au_manufacturer]);
                ("au".to_string(), Vec::from(bytes))
            }
            PluginInfo::Vst { id } => {
                let bytes: [u8; 4] = transmute!(id);
                ("vst".to_string(), Vec::from(bytes))
            }
            PluginId::Vst3 { guid } => {
                let bytes: [u8; 16] = transmute!(guid);
                ("vst3".to_string(), Vec::from(bytes))
            }
        };
        Ok(PluginRow {
            plugin_type,
            id,
            name,
            vendor,
        })
    }
}

impl TryFrom<PluginRow> for Plugin {
    type Error = TempoError;

    fn try_from(value: PluginRow) -> Result<Self> {
        let PluginRow {
            plugin_type,
            id,
            name,
            vendor,
        } = value;

        match plugin_type.as_str() {
            "au" => {
                let bytes: [u8; 12] = id.try_into().map_err(|e: Vec<u8>| TempoError::Db(format!("Could not parse PluginRow, expected 12-byte id for au, found {} bytes instead", e.len())))?;
                let values: [u32; 3] = transmute!(bytes);
                Ok(Plugin {
                    info: PluginId::Au {
                        au_type: values[0],
                        au_subtype: values[1],
                        au_manufacturer: values[2],
                    },
                    name: Some(name),
                    vendor: Some(vendor),
                })
            }
            "vst" => {
                let bytes: [u8; 4] = id.try_into().map_err(|e: Vec<u8>| TempoError::Db(format!("Could not parse PluginRow, expected 4-byte id for vst, found {} bytes instead", e.len())))?;
                let id = transmute!(bytes);
                Ok(Plugin {
                    info: PluginId::Vst { id },
                    name: Some(name),
                    vendor: Some(vendor),
                })
            }
            "vst3" => {
                let bytes: [u8; 16] = id.try_into().map_err(|e: Vec<u8>| TempoError::Db(format!("Could not parse PluginRow, expected 16-byte id for vst3, found {} bytes instead", e.len())))?;
                let guid: [u32; 4] = transmute!(bytes);
                Ok(Plugin {
                    info: PluginId::Vst3 { guid },
                    name: Some(name),
                    vendor: Some(vendor),
                })
            }
            t => Err(TempoError::Ableton(format!(
                "Could not parse PluginRow, found unknown type {}",
                t
            ))),
        }
    }
}


impl TryFrom<PluginRow> for ScannedPlugin {
    type Error = TempoError;

    fn try_from(value: PluginRow) -> std::result::Result<Self, Self::Error> {
        let PluginRow {
            plugin_type,
            id,
            name,
            vendor,
        } = value;

        let plugin_type = match plugin_type.as_str() {
            "au" => PluginType::Au,
            "vst" => PluginType::Vst,
            "vst3" => PluginType::Vst3,
            t => {
                return Err(TempoError::Plugin(format!(
                    "Could not build ScannedPlugin from PluginRow, unknown plugin type {t}"
                )))
            }
        };

        Ok(ScannedPlugin {
            plugin_type,
            name,
            vendor,
        })
    }
}
*/

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
    log::warn!("called open_settings() on non-macOS");
}

/// Returns true if we have full disk access
#[cfg(target_os = "macos")]
pub fn check_full_disk() -> Result<bool> {
    // macOS has no api for telling whether we have full disk access directly
    // there are some directories we can try reading to tell whether we have access or not
    // based off of https://github.com/MacPaw/PermissionsKit/blob/master/PermissionsKit/Private/FullDiskAccess/MPFullDiskAccessAuthorizer.m

    // this can fail if directories::UserDirs::new() fails for some reason

    let dirs = directories::UserDirs::new().ok_or(TempoError::Other(
        "Failed to find home directory when checking for Full Disk Access".into(),
    ))?;
    let home_dir = dirs.home_dir();

    let test_paths: [PathBuf; 4] = [
        home_dir.join("Library/Safari/CloudTabs.db"),
        home_dir.join("Library/Safari/Bookmarks.plist"),
        home_dir.join("Library/Application Support/com.apple.TCC/TCC.db"),
        PathBuf::from("/Library/Preferences/com.apple.TimeMachine.plist"),
    ];

    for p in test_paths {
        if std::fs::File::open(p).is_ok() {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(not(target_os = "macos"))]
pub fn check_full_disk() -> Result<bool> {
    Ok(true)
}

pub fn new_ulid() -> String {
    ulid::Ulid::new().to_string()
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
        TempoError::File(format!(
            "Failed to open file {} to calculate hash, error: {e}",
            path_to_str(file)
        ))
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
        .ok_or(TempoError::File(format!(
            "File {} does not have a filename",
            path.to_string_lossy()
        )))?
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

pub fn is_valid_ulid(s: &str) -> bool {
    s.len() == 26 && s.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Extracts the first two characters from a ulid's hash part
pub fn get_ulid_chars(ulid: &str) -> String {
    ulid[10..=11].into()
}
