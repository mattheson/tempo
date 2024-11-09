
use crate::{
    folder::FolderInner, misc::{Result, TempoError}, note::Note, shared::NewNote, tempo::Tempo, types::ChannelDoc
};
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct ChannelInfo {
    pub channel_ulid: String,
    pub doc: ChannelDoc,
}

/// Represents a channel in a folder
#[derive(Debug)]
pub struct Channel {
    tempo: Tempo,
    // emitter: StateEmitter,

    folder: FolderInner,
    inner: ChannelInner,
}

#[derive(Debug, Clone)]
pub struct ChannelInner(Arc<RwLock<Option<ChannelInfo>>>);

impl ChannelInner {
    pub fn ulid(&self) -> Option<String> {
        let channel = self.0.read().unwrap();
        channel.as_ref().map(|info| info.channel_ulid.clone())
    }
}

impl Channel {
    pub fn create(tempo: Tempo, folder: FolderInner, channel_name: &str) -> Result<Self> {
        let (channel_ulid, doc) =
            ChannelDoc::create(&folder.path()?, &folder.username()?, channel_name)?;

        Ok(Self {
            tempo,
            // emitter,
            folder,
            inner: ChannelInner(Arc::new(RwLock::new(Some(ChannelInfo {
                channel_ulid,
                doc,
            })))),
        })
    }

    pub fn load(
        tempo: Tempo,
        folder: FolderInner,
        channel_ulid: Option<&str>,
    ) -> Result<Self> {
        Ok(Self {
            tempo,
            inner: ChannelInner(Arc::new(RwLock::new(match channel_ulid {
                None => None,
                Some(channel_ulid) => {
                    let doc = ChannelDoc::load(&folder.path()?, &folder.username()?, channel_ulid)?;

                    Some(ChannelInfo {
                        channel_ulid: channel_ulid.into(),
                        doc,
                    })
                }
            }))),
            // emitter,
            folder,
        })
    }

    // pub fn ulid(&self) -> Option<String> {
    //     self.inner.ulid()
    // }

    pub fn note(&self, note_ulid: &str) -> Result<Note> {
        Note::load(
            self.tempo.clone(),
            self.folder.clone(),
            self.inner.clone(),
            note_ulid,
            // self.emitter.clone(),
        )
    }

    pub fn create_note(&self, note: NewNote) -> Result<Note> {
        Note::create(
            self.tempo.clone(),
            self.folder.clone(),
            self.inner.clone(),
            note,
            // self.emitter.clone(),
        )
    }

    pub fn get(&self) -> Result<ChannelDoc> {
        if let Some(c) = self.inner.0.read().unwrap().as_ref() {
            Ok(c.doc.clone())
        } else {
            Err(TempoError::Channel("Tried to get doc of global channel".into()))
        }
    }

    // for testing
    #[allow(dead_code)]
    pub fn ulid(&self) -> Option<String> {
        self.inner.ulid()
    }
}

// pub fn iter_notes(&self) -> Result<impl Iterator<Item = Note>> {
//     let folder = self.folder.clone();
//     let emitter = self.emitter.clone();
//     let this = self.inner.clone();

//     let channel_ulid = self.inner.ulid().map(|u| u.to_owned());

//     Ok(
//         iter_notes(folder.path()?, channel_ulid.as_deref())?.filter_map(
//             move |(_, note_ulid)| match Note::load(
//                 folder.clone(),
//                 this.clone(),
//                 &note_ulid,
//                 emitter.clone(),
//             ) {
//                 Ok(n) => Some(n),
//                 Err(e) => {
//                     error!(
//                         "iter_notes(): failed to load Note {note_ulid} in {}, error: {e}",
//                         path_to_str(folder.path())
//                     );
//                     None
//                 }
//             },
//         ),
//     )
// }

// pub fn get(&self) -> Result<Channel> {
//     if let Some(ChannelInfo { channel_ulid, doc }) = self.inner.0.as_ref() {
//         self.emitter.emit_channel(ChannelEmit {
//             folder: self.folder.path().to_path_buf(),
//             channel_ulid: channel_ulid.clone(),
//             doc: doc.clone(),
//         })
//     } else {
//         // when inner is None we are the global channel
//         // global channel has no metadata so we don't emit anything
//         Ok(())
//     }
// }
