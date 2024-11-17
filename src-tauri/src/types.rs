use std::collections::HashMap;
use std::path::PathBuf;

use autosurgeon::Reconcile;

// /// Metadata about a session.
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct SessionData {
//     // name of the session
//     pub name: String,

//     // { username : client uuid }
//     pub users: HashMap<String, uuid::Uuid>,

//     pub channels: HashMap<ulid::Ulid, ChannelInfo>,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct ChannelInfo {
//     pub name: String,
//     pub creator: uuid::Uuid,
//     pub hidden: bool,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct NoteData {
//     pub sender: uuid::Uuid,

//     pub body: String,
//     pub parents: Vec<ulid::Ulid>,

//     pub attachment: Option<Attachment>,

//     pub comments: HashMap<ulid::Ulid, RepliableComment>,

//     pub hidden: bool,
// }

// /// Repliable comment on a note.
// /// Only one level of replies is supported.
// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct RepliableComment {
//     pub comment: Comment,

//     pub replies: HashMap<ulid::Ulid, Comment>,
// }

// #[derive(
//     Debug, serde::Serialize, serde::Deserialize, autosurgeon::Reconcile, autosurgeon::Hydrate,
// )]
// pub struct Comment {
//     pub sender: uuid::Uuid,
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

// /// A request from the frontend to make a new note.
// #[derive(Debug, serde::Serialize, serde::Deserialize)]
// pub struct NewNote {
//     pub body: String,
//     pub parents: Vec<ulid::Ulid>,
//     pub attachment: Option<NewAttachment>,
// }

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

// id types
// we serialize to bytes

pub struct TempoUlid(ulid::Ulid);

impl TempoUlid {
    pub fn new() -> Self {
        Self(ulid::Ulid::new())
    }
}

impl serde::Serialize for TempoUlid {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct TempoUlidVisitor;

impl<'de> serde::de::Visitor<'de> for TempoUlidVisitor {
    type Value = TempoUlid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ulid in byte format (16 bytes)")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(TempoUlid(ulid::Ulid::from_bytes(v.try_into().map_err(
            |_| E::custom(format!("found {} bytes for ulid, expected 16", v.len())),
        )?)))
    }
}

impl<'de> serde::Deserialize<'de> for TempoUlid {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(TempoUlidVisitor)
    }
}

impl autosurgeon::Hydrate for TempoUlid {
    fn hydrate_bytes(_bytes: &[u8]) -> Result<Self, autosurgeon::HydrateError> {
        Ok(TempoUlid(ulid::Ulid::from_bytes(
            _bytes.try_into().map_err(|_| {
                autosurgeon::HydrateError::unexpected(
                    "16 bytes for ulid",
                    format!("{} bytes", _bytes.len()),
                )
            })?,
        )))
    }
}

impl autosurgeon::Reconcile for TempoUlid {

}

pub struct TempoUuid(uuid::Uuid);

impl TempoUuid {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}
