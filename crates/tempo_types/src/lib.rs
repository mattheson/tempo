use std::collections::HashMap;

use tempo_id::{Ulid, Uuid};


// #[derive(Debug, serde::Serialize)]
// pub struct SessionData {
//     // name of the session
//     pub name: String,
//     pub users: HashMap<String, Uuid>,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub enum Note {
//     Channel(),
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub struct ChannelDesc {
//     pub name: String
// }

// /// Contains users
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct SessionData {
//     // name of the session
//     pub name: String,
//     pub users: HashMap<String, Uuid>,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct ChannelInfo {
//     pub creator: Uuid,

//     pub name: String,

//     pub hidden: bool,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct NoteData {
//     pub sender: Uuid,

//     pub body: String,
//     pub parents: Vec<Ulid>,

//     pub attachment: Option<Attachment>,

//     #[autosurgeon(with = "autosurgeon::map_with_parseable_keys")]
//     pub comments: HashMap<Ulid, RepliableComment>,

//     pub hidden: bool,
// }

// /// Repliable comment on a note.
// /// Only one level of replies is supported.
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct RepliableComment {
//     pub comment: Comment,

//     #[autosurgeon(with = "autosurgeon::map_with_parseable_keys")]
//     pub replies: HashMap<Ulid, Comment>,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct Comment {
//     pub sender: Uuid,
//     pub body: String,

//     pub hidden: bool,
// }

// /// An attachment on a note.
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub enum Attachment {
//     AbletonProject(AbletonProjectAttachment),
//     Audio(AudioAttachment),
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct AbletonProjectAttachment {
//     pub title: String,

//     // hash of .als
//     pub hash: String,

//     // hash of render of project file
//     pub render_hash: Option<String>,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct AudioAttachment {
//     pub title: String,
//     pub hash: String,
// }

/// A request from the frontend to make a new note.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NewNote {
    pub body: String,
    pub parents: Vec<Ulid>,
    pub attachment: Option<NewAttachment>,
}

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub enum NewAttachment {
//     AbletonProject(NewAbletonProjectAttachment),
//     Audio(NewAudioAttachment),
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub struct NewAbletonProjectAttachment {
//     pub title: String,
//     pub path: std::path::PathBuf,
//     pub render: Option<std::path::PathBuf>,
// }

// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub struct NewAudioAttachment {
//     pub title: String,
//     pub path: std::path::PathBuf,
// }

// /// A request from the frontend to create a comment on a note.
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct NewComment {
//     // this ulid is a key within the note's comments map
//     pub reply_ulid: Option<String>,

//     pub body: String,
// }
