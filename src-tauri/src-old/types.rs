// automerge and json types
// these are saved to disk and stored in Tempo folders

use std::collections::HashMap;

use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::daw::{ableton::AbletonPluginRef, project::ProjectType};

// TODO figure out how to disable ts_rs generated tests when running `cargo test`
// not too big of a deal though

/// Stores all the metadata in a folder
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct MetaDoc {
    // { install ulid : username }
    pub users: HashMap<String, String>
}

/// Stores metadata about a channel.
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct ChannelInfo {
    pub name: String,
    pub creator: String,
    pub hidden: bool,
}

/// A single note in a Tempo folder.
/// Notes are stored within channels.
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct NoteDoc {
    pub sender: String,
    pub body: String,
    pub reply_ulid: Option<String>, // ulid of note being replied to, has to be in same channel

    // a note can only have one attachment
    // this helps simplify Tempo's version management
    // a user replies to a message with a project to represent a new version of a project
    // TODO maybe it should be possible to reply to multiple messages for 'merging' of projects
    pub attachment: Option<Attachment>,

    // { ulid : comment }
    // ulid is just used to make it easy to sort these
    // you can only comment on notes which have project/audio attachments for now
    // comments will be ignored on notes with no attachments
    pub comments: HashMap<String, RepliableComment>,
}

/// An attachment on a note.
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub enum Attachment {
    Project(ProjectAttachment),
    Audio(AudioAttachment),
}

#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct ProjectAttachment {
    pub title: String,

    // project type and filename can be read from corresponding `FileInfo``
    pub hash: String,

    // hash of render of project file
    pub render_hash: Option<String>,
}

#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct AudioAttachment {
    pub title: Option<String>,
    pub hash: String,
}

/// Repliable comment on a note.
/// Only one level of replies is supported.
#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct RepliableComment {
    pub comment: Comment,
    // { ulid : comment }
    pub replies: HashMap<String, Comment>,
}

#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS)]
pub struct Comment {
    pub sender: String,
    pub body: String,
}

/// Contains metadata about a file.
/// This is immutable and created when a file is initially added to a folder.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub filename: String,
    pub added_by: String,
    pub timestamp: u64,
    pub meta: FileMeta,
}

/// File's type and any extra type-specific info
#[derive(Debug, Serialize, Deserialize)]
pub enum FileMeta {
    Audio(AudioType),

    Project(ProjectData),

    MaxForLive,

    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProjectData {
    Ableton {
        // this hashmap stores file references
        // these files are copied into the Files directory of the live project when copied out of Tempo

        // TODO it's possible that a project references two different copies of the same sample
        // i am just adjusting the filerefs to point at one copy of the sample
        // maybe could cause problems but don't see how for now

        // { file hash : filename to use in Files folder }
        refs: HashMap<String, String>,

        // plugins used in this project
        plugins: Vec<AbletonPluginRef>,
    },
}

/// All file types known by Tempo.
#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub enum FileType {
    // always AudioType::Other for now
    // tauri supports loading mime type
    Audio(AudioType),

    // project file
    Project(ProjectType),
    MaxForLive,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
pub enum AudioType {
    Wav,
    Mp3,
    Flac,
    Other,
}
