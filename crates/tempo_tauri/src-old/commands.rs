use crate::{
    misc::{self, fatal_error_close_windows, path_to_str},
    shared::*,
    structure::validate_folder_structure,
    tempo::Tempo,
    types::{Attachment, ChannelDoc},
};
use log::info;
use std::{path::PathBuf, sync::LazyLock};
use tauri::{AppHandle, Manager};

type Result<T> = std::result::Result<T, BackendError>;

// TODO this macro is just because i dont want to type the lifetime every time
// possibly remove
macro_rules! St {
    () => {
        tauri::State<'_, Tempo>
    }
}

fn err<T>(s: String) -> Result<T> {
    Err(BackendError::Other(s))
}

// commands ------------------------------------------------------------------------------------
// ensure you keep this in sync with commands.ts in `src`

#[tauri::command]
pub async fn get_store_path(state: St!()) -> Result<PathBuf> {
    Ok(state.get_store_path())
}

#[tauri::command]
pub async fn need_full_disk(handle: AppHandle) -> Result<bool> {
    Ok(match crate::misc::check_full_disk() {
        Ok(b) => !b,
        Err(e) => {
            fatal_error_close_windows(
                &handle,
                &format!("error while checking for full disk access: {}", e),
            );
        }
    })
}

/// Opens full disk setting in macOS settings
#[tauri::command]
pub async fn open_full_disk() -> Result<()> {
    misc::open_full_disk();
    Ok(())
}

#[tauri::command]
pub async fn restart(handle: AppHandle) {
    tauri::process::restart(&handle.env());
}

#[tauri::command]
pub fn fatal(msg: String, handle: AppHandle) {
    fatal_error_close_windows(&handle, &msg);
}

#[tauri::command]
pub async fn verify_user_has_ableton() -> Result<bool> {
    Ok(crate::daw::ableton::have_plugin_db()?)
}

#[tauri::command]
pub async fn scan_folders(state: St!()) -> Result<Vec<FolderInfo>> {
    Ok(state.scan_folders())
}

#[tauri::command]
pub async fn scan_folder(folder: PathBuf, state: St!()) -> Result<FolderInfo> {
    Ok(state.scan_folder(&folder)?)
}

// TODO
// sometimes multiple get_folder_datas() running at the same time results in corruption
// this corruption involves empty automerge docs being saved to disk
// not completely sure what the cause of this is yet, probably something to do with overwriting of unchanged automerge docs?
// have made some adjustments in `automerge.rs` to help with this
// optimally corruption would not happen even if multiple tasks are loading docs
static FOLDER_DATA_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

/// Sends all data stored in a folder to frontend.
#[tauri::command]
pub async fn get_folder_data(state: St!(), folder: PathBuf) -> Result<FolderData> {
    let _lock = FOLDER_DATA_LOCK.lock().await;

    Ok(state.folder(&folder)?.get_data()?)
}

/// Checks whether the given folder is inside of an existing Tempo folder.
/// `Some(folder)` if it is, `None` otherwise.
#[tauri::command]
pub async fn check_folder_inside_folder(folder: PathBuf) -> Option<PathBuf> {
    let mut folder = folder.parent().map(|parent| parent.to_owned())?;

    // walk up dirs and check if they're tempo dirs
    // to prevent users from creating folders inside of folders

    loop {
        match validate_folder_structure(&folder) {
            Ok(()) => {
                info!("{} is a tempo folder", path_to_str(&folder));
                return Some(folder);
            }
            Err(e) => {
                info!(
                    "failed to check if {} is a tempo folder, reason: {e}",
                    path_to_str(&folder)
                );
            }
        }
        folder = match folder.parent() {
            Some(parent) => parent.to_owned(),
            None => return None,
        };
    }
}

#[tauri::command]
pub async fn is_username_free(folder: PathBuf, username: String, state: St!()) -> Result<bool> {
    Ok(state.is_username_free(&folder, &username)?)
}

/// Rebuilds Tempo's shared database AND copies it into all valid folders.
#[tauri::command]
pub async fn scan_plugins(state: St!()) -> Result<()> {
    state.scan_plugins()?;
    Ok(state.copy_db()?)
}

#[tauri::command]
pub async fn get_last_plugin_scan_time(state: St!()) -> Result<Option<i64>> {
    Ok(state.get_last_plugin_scan_time()?)
}

/// Takes a path to a folder.
/// Adds the folder to Tempo if it's an existing shared folder.
/// Otherwise sets up a new folder and adds it.
/// Fails if a user is already using the provided username.
#[tauri::command]
pub async fn create_or_add_folder(folder: PathBuf, username: String, state: St!()) -> Result<()> {
    if !folder.exists() {
        return err(format!(
            "Folder {} does not exist",
            folder.to_string_lossy()
        ));
    } else if !folder.is_dir() {
        return err(format!("{} is not a directory", folder.to_string_lossy()));
    } else if validate_folder_structure(&folder).is_ok() {
        info!("adding existing folder {}", folder.to_string_lossy());
        state.add_folder(&folder, &username)?
    } else {
        info!("creating new tempo folder in {}", folder.to_string_lossy());
        state.create_folder(&folder)?;
        state.add_folder(&folder, &username)?
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_folder(folder: PathBuf, state: St!()) -> Result<()> {
    Ok(state.remove_folder(&folder)?)
}

#[tauri::command]
pub async fn create_channel(
    folder: PathBuf,
    channel_name: String,
    state: St!(),
) -> Result<ChannelDoc> {
    Ok(state
        .folder(&folder)?
        .create_channel(&channel_name)?
        .get()?)
}

#[tauri::command]
pub async fn get_attachment_type(file: PathBuf) -> Result<AttachmentType> {
    Ok(AttachmentType::get(&file)?)
}

#[tauri::command]
pub async fn create_note(
    folder: PathBuf,
    channel_ulid: Option<String>,
    note: NewNote,
    state: St!(),
) -> Result<TempoResult<SharedNote>> {
    info!("got msg {:#?}", &note);
    Ok(state
        .folder(&folder)?
        .channel(channel_ulid.as_deref())?
        .create_note(note)?
        .get()?)
}

#[tauri::command]
pub async fn add_comment(
    folder: PathBuf,
    channel_ulid: Option<String>,
    note_ulid: String,
    comment: NewComment,
    state: St!(),
) -> Result<TempoResult<SharedNote>> {
    Ok(state
        .folder(&folder)?
        .channel(channel_ulid.as_deref())?
        .note(&note_ulid)?
        .add_comment(comment)?
        .get()?)
}

/// Returns path to new copy of project
/// `dir` is directory where project will be copied into
#[tauri::command]
pub async fn copy_project(
    folder: PathBuf,
    channel_ulid: Option<String>,
    note_ulid: String,
    dest_dir: PathBuf,
    state: St!(),
) -> Result<(PathBuf, Vec<FileErr>)> {
    match state
        .folder(&folder)?
        .channel(channel_ulid.as_deref())?
        .note(&note_ulid)?
        .attachment()
    {
        None => err("Cannot copy project from a note containing no project".into()),
        Some(a) => match a {
            Attachment::Project(p) => Ok(p.copy(&folder, &dest_dir)?),
            Attachment::Audio(_) => {
                err("Cannot copy project from a note with an audio attachment".into())
            }
        },
    }
}

#[tauri::command]
pub async fn get_file_info(
    folder: PathBuf,
    file_sha256: String,
    state: St!(),
) -> Result<SharedFileInfo> {
    Ok(SharedFileInfo::from(
        state.folder(&folder)?.file_info(&file_sha256)?,
    ))
}

#[tauri::command]
pub async fn scan_project_file_refs(project: PathBuf) -> Result<ProjectFileRefScan> {
    Ok(ProjectFileRefScan::new(&project)?)
}

#[tauri::command]
pub async fn scan_project_plugins(
    folder: PathBuf,
    project: PathBuf,
    state: St!(),
) -> Result<PluginScan> {
    Ok(state.folder(&folder)?.scan_project_plugins(&project)?)
}
