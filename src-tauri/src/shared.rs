// Types directly shared between frontend and backend

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use ts_rs::TS;

use crate::{
    daw::{plugin::PluginType, project::ProjectType},
    misc::TempoError,
    types::{AudioType, ChannelDoc, FileInfo, FileMeta, FileType, ProjectData, RepliableComment},
};

/// Error enum that's directly shared with the frontend.
#[derive(Error, Debug, Serialize, TS)]
#[ts(export)]
pub enum BackendError {
    #[error("Backend error: {0}")]
    Other(String),

    #[error("Internal Tempo error: {0}")]
    TempoError(String),

    #[error("Tauri error: {0}")]
    Tauri(String),
}

// TODO maybe could just use regular result instead of this

#[derive(Error, Debug, Serialize, TS)]
#[ts(export)]
pub enum TempoResult<T> {
    Ok(T),
    Err(String),
}

impl<T> From<TempoError> for TempoResult<T> {
    fn from(value: TempoError) -> Self {
        Self::Err(value.to_string())
    }
}

impl From<TempoError> for BackendError {
    fn from(value: TempoError) -> Self {
        BackendError::TempoError(value.to_string())
    }
}

impl From<tauri::Error> for BackendError {
    fn from(value: tauri::Error) -> Self {
        BackendError::Tauri(value.to_string())
    }
}

/// Path of a folder and its validity
#[derive(Serialize, TS)]
#[ts(export)]
pub struct FolderInfo {
    pub path: PathBuf,
    pub error: Option<String>, // reason for folder invalidity, if any
}

/// All data stored in a folder. Sent to the frontend.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct FolderData {
    pub username: String,

    // { ulid : doc }
    pub global: HashMap<String, TempoResult<SharedNote>>,

    // { ulid : (doc, { ulid : doc })}
    pub channels: HashMap<String, ChannelData>,
}

/// All data stored in a folder. Sent to the frontend.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ChannelData {
    pub meta: TempoResult<ChannelDoc>,
    pub notes: HashMap<String, TempoResult<SharedNote>>,
}

/// Similar to `NoteDoc`, but contains extra information validating note's attachment (if any)
#[derive(Serialize, TS)]
#[ts(export)]
pub struct SharedNote {
    pub sender: String,
    pub body: String,
    pub reply_ulid: Option<String>,

    pub attachment: Option<SharedAttachment>,

    pub comments: HashMap<String, RepliableComment>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub enum SharedAttachment {
    Project(SharedProjectAttachment),
    Audio(SharedAudioAttachment),
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SharedProjectAttachment {
    pub title: String,
    pub project: TempoResult<ProjectInfo>, // err if we can't load FileInfo
    pub render: Option<TempoResult<AudioFileInfo>>, // err if render file or its FileInfo is missing
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct SharedAudioAttachment {
    pub title: Option<String>,
    pub file: TempoResult<AudioFileInfo>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct AudioFileInfo {
    // path here for playback
    pub path: PathBuf,
    pub filename: String,
}

/// Info about a project that's been added to a Tempo folder.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct ProjectInfo {
    pub filename: String,
    pub data: SharedProjectData,
}

/// Scanned information about a project in a a Tempo Folder
#[derive(Serialize, TS)]
#[ts(export)]
pub enum SharedProjectData {
    Ableton {
        missing_files: Vec<String>,

        // plugins user doesn't have installed
        missing_plugins: Vec<PluginRef>,
    },
}

/// A request from the frontend to make a new note.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct NewNote {
    pub body: String,
    pub reply_ulid: Option<String>,
    pub attachment: Option<NewAttachment>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub enum NewAttachment {
    Project(NewProjectAttachment),
    Audio(NewAudioAttachment),
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct NewProjectAttachment {
    pub title: String,
    pub path: PathBuf,
    pub render: Option<PathBuf>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct NewAudioAttachment {
    pub title: Option<String>,
    pub path: PathBuf,
}

/// A request from the frontend to create a comment on a note.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct NewComment {
    // this ulid is a key within the note's comments map
    pub reply_ulid: Option<String>,

    pub body: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub enum AttachmentType {
    Audio(AudioType),
    Project(ProjectType),
}

/// Used when adding a project to a folder.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ProjectFileRefScan {
    pub ok: HashSet<FileRef>,
    pub missing: HashSet<MissingFileRef>,
}

/// A scan of all plugins inside of a project file and a listing of plugins that clients are missing.
/// Used when adding a project to a folder.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct PluginScan {
    // all plugins found in project
    pub plugins: Vec<PluginRef>,
    // { username: idxs of missing plugins }
    pub missing: HashMap<String, Vec<usize>>,
}

#[derive(Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct FileRef {
    // TODO: this is based on Ableton's filerefs
    // might be appropriate to make rel optional? or just have this be an absolute path always
    // or actually send AbletonFileRefs instead
    pub rel: String,
    pub abs: String,
}

#[derive(Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct MissingFileRef {
    pub file: FileRef,
    pub err: String,
}

#[derive(Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct FileErr {
    pub filename: String,
    pub err: String,
}

#[derive(Eq, PartialEq, Hash, Debug, Serialize, TS)]
#[ts(export)]
pub struct PluginRef {
    pub plugin_type: PluginType,
    pub name: String,
    pub vendor: String,
}

/// A stripped down `FileInfo`
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SharedFileInfo {
    pub filename: String,
    pub added_by: String,
    pub timestamp: u64,
    pub file_type: FileType,
}

impl From<FileInfo> for SharedFileInfo {
    fn from(value: FileInfo) -> Self {
        let FileInfo {
            filename,
            added_by,
            timestamp,
            meta,
        } = value;

        Self {
            filename,
            added_by,
            timestamp,
            file_type: match meta {
                FileMeta::Audio(t) => FileType::Audio(t),
                FileMeta::Project(d) => match d {
                    ProjectData::Ableton {
                        refs: _,
                        plugins: _,
                    } => FileType::Project(ProjectType::Ableton),
                },
                FileMeta::MaxForLive => FileType::MaxForLive,
                FileMeta::Other => FileType::Other,
            },
        }
    }
}
