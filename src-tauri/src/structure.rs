// creating, validating and adding files to Tempo folders

use std::{
    fs,
    path::{Path, PathBuf},
};

use log::{error, warn};
use walkdir::WalkDir;

use crate::misc::{get_filename, get_ulid_chars, is_valid_ulid, path_to_str, Result, TempoError};

/// Generates a set of required subdirectories in a Tempo folder.
pub fn get_tempo_dirs(folder: &Path) -> Vec<PathBuf> {
    let tempo_dir = folder.join("tempo");
    let channels_dir = tempo_dir.join("channels");
    let files_dir = tempo_dir.join("files");
    let clients_dir = tempo_dir.join("clients");
    let global_channel = channels_dir.join("global");

    vec![
        tempo_dir,
        channels_dir,
        files_dir,
        clients_dir,
        global_channel,
    ]
}

/// Creates a new Tempo folder in the given directory. Creates the directory if needed.
pub fn create_tempo_folder(folder: &Path) -> Result<()> {
    if !folder.is_dir() {
        return Err(TempoError::Folder(format!(
            "{} exists and is not a folder",
            path_to_str(folder)
        )));
    }

    if !folder.exists() {
        fs::create_dir_all(folder)?;
    }

    for d in get_tempo_dirs(folder) {
        fs::create_dir_all(d)?;
    }

    Ok(())
}

pub fn validate_folder_structure(folder: &Path) -> std::result::Result<(), String> {
    if !folder.exists() {
        return Err("Folder does not exist".into());
    }

    if !folder.is_dir() {
        return Err("Path is not a directory, found file instead".into());
    }

    let mut missing_dirs: Vec<String> = vec![];

    for d in get_tempo_dirs(folder) {
        let exists = match fs::exists(&d) {
            Ok(e) => e,
            Err(e) => return Err(format!("Failed to scan folder, ensure you have proper read/write permissions in the folder {}, error: {e}", path_to_str(folder)))
        };
        if !exists || !d.is_dir() {
            // get_filename() shouldnt fail here
            missing_dirs.push(get_filename(&d)?)
        }
    }

    if !missing_dirs.is_empty() {
        return Err(format!(
            "Folder appears to be an invalid Tempo folder or corrupt, missing the following required subdirectories: {}",
            missing_dirs
                .into_iter()
                .fold(String::new(), |s, d| s + &d + " ")
        ));
    }

    Ok(())
}

pub fn expect_valid_folder(folder: &Path) -> Result<()> {
    validate_folder_structure(folder).map_err(TempoError::InvalidFolder)
}

pub fn get_channel_path(folder: &Path, channel_ulid: &str) -> PathBuf {
    folder
        .join("tempo")
        .join("channels")
        .join(get_ulid_chars(channel_ulid))
        .join(channel_ulid)
}

pub fn get_global_channel_path(folder: &Path) -> PathBuf {
    folder.join("tempo").join("channels").join("global")
}

pub fn get_channel_meta_path(folder: &Path, channel_ulid: &str) -> PathBuf {
    get_channel_path(folder, channel_ulid).join("meta")
}

pub fn get_note_path(folder: &Path, channel_ulid: Option<&str>, note_ulid: &str) -> PathBuf {
    match channel_ulid {
        Some(channel_ulid) => get_channel_path(folder, channel_ulid),
        None => get_global_channel_path(folder),
    }
    .join(get_ulid_chars(note_ulid))
    .join(note_ulid)
}

pub fn get_file_dir_path(folder: &Path, file_sha256: &str) -> PathBuf {
    folder
        .join("tempo")
        .join("files")
        .join(&file_sha256[0..=1])
        .join(file_sha256)
}

pub fn get_file_meta_path(folder: &Path, file_sha256: &str) -> PathBuf {
    get_file_dir_path(folder, file_sha256).join("meta")
}

pub fn get_file_path(folder: &Path, file_sha256: &str) -> PathBuf {
    get_file_dir_path(folder, file_sha256).join("file")
}

pub fn get_clients_path(folder: &Path) -> PathBuf {
    folder.join("tempo").join("clients")
}

pub fn get_client_dir_path(folder: &Path, username: &str) -> PathBuf {
    get_clients_path(folder).join(username)
}

pub fn get_client_shared_db_path(folder: &Path, username: &str) -> PathBuf {
    get_client_dir_path(folder, username).join("shared.sqlite")
}

fn exists_with_nice_error(folder: &Path, res: std::io::Result<bool>) -> Result<bool> {
    match res {
        Ok(e) => Ok(e),
        Err(e) => Err(TempoError::Folder(format!("Failed to read files in folder {:#?}. Ensure the folder exists and has proper read/write permissions. Error: {e}", path_to_str(folder))))
    }
}

// pub fn is_username_taken(folder: &Path, username: &str) -> Result<bool> {
//     exists_with_nice_error(
//         folder,
//         fs::exists(get_client_shared_db_path(folder, username)),
//     )
// }

pub fn channel_exists(folder: &Path, channel_ulid: &str) -> Result<bool> {
    exists_with_nice_error(folder, fs::exists(get_channel_path(folder, channel_ulid)))
}

pub fn note_exists(folder: &Path, channel_ulid: Option<&str>, note_ulid: &str) -> Result<bool> {
    exists_with_nice_error(
        folder,
        fs::exists(get_note_path(folder, channel_ulid, note_ulid)),
    )
}

pub fn file_exists(folder: &Path, file_sha256: &str) -> Result<bool> {
    exists_with_nice_error(folder, fs::exists(get_file_dir_path(folder, file_sha256).join("file")))
}

// pub fn file_meta_exists(folder: &Path, file_sha256: &str) -> Result<bool> {
//     exists_with_nice_error(folder, fs::exists(get_file_meta_path(folder, file_sha256)))
// }

/// Iterates over a directory which holds ulids.
/// Takes a the directory which holds the 2-character dirs (which themselves contain ulid-named dirs).
fn iter_ulid(dir: PathBuf) -> Result<impl Iterator<Item = (PathBuf, String)>> {
    Ok(WalkDir::new(&dir)
        .min_depth(2)
        .max_depth(2)
        .into_iter()
        .filter_map(move |entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if !path.is_dir() {
                    // spams log
                    // warn!("found file in dir: {}, ignoring", path.to_string_lossy());
                    None
                } else {
                    match path.file_name() {
                        Some(f) => {
                            let filename = f.to_string_lossy();
                            if is_valid_ulid(&filename) {
                                Some((path.to_path_buf(), filename.to_string()))
                            } else {
                                // spams log
                                // warn!("found non-uuid dir {}", path_to_str(path));
                                None
                            }
                        }
                        None => {
                            warn!("found file with no filename: {}", path_to_str(path));
                            None
                        }
                    }
                }
            }
            Err(e) => {
                warn!(
                    "error while reading folder entry in {}, {}",
                    path_to_str(&dir),
                    e
                );
                None
            }
        }))
}

/// Iterates over channels in a folder.
/// Returns an iterator of (path to channel, channel ulid)
pub fn iter_channels(folder: &Path) -> Result<impl Iterator<Item = (PathBuf, String)>> {
    expect_valid_folder(folder)?;

    let channels_dir = folder.join("tempo").join("channels");

    iter_ulid(channels_dir)
}

/// Iterates over notes stored in a channel.
/// Returns an iterator of (path to note, note ulid)
pub fn iter_notes(
    folder: &Path,
    channel_ulid: Option<&str>,
) -> Result<impl Iterator<Item = (PathBuf, String)>> {
    expect_valid_folder(folder)?;

    let channel_dir = match channel_ulid {
        Some(channel_ulid) => {
            if !channel_exists(folder, channel_ulid)? {
                return Err(TempoError::Channel(format!(
                    "Channel {channel_ulid} does not exist"
                )));
            }
            get_channel_path(folder, channel_ulid)
        }
        None => get_global_channel_path(folder),
    };

    iter_ulid(channel_dir)
}

pub fn iter_clients(folder: &Path) -> Result<impl Iterator<Item = (PathBuf, String)>> {
    expect_valid_folder(folder)?;

    let folder = folder.to_path_buf();

    Ok(get_clients_path(&folder)
        .read_dir()?
        .filter_map(move |e| match e {
            Ok(e) => Some(e),
            Err(e) => {
                error!(
                    "iter_clients(): failed to get DirEntry in clients in {}, error: {e}",
                    path_to_str(&folder)
                );
                None
            }
        })
        .filter_map(|e| {
            let path = e.path();
            match get_filename(&path) {
                Ok(f) => Some((path, f)),
                Err(e) => {
                    error!(
                        "iter_clients(): failed to get directory name for {}, error: {e}",
                        path_to_str(&path)
                    );
                    None
                }
            }
        }))
}
