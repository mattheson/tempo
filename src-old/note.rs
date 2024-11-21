use std::{collections::HashMap, path::Path};

use log::error;

use crate::{
    channel::ChannelInner,
    db::SharedDb,
    file::add_file,
    folder::FolderInner,
    misc::{new_ulid, path_to_str, Result, TempoError},
    shared::{
        AudioFileInfo, NewAttachment, NewAudioAttachment, NewComment, NewNote,
        NewProjectAttachment, PluginRef, ProjectInfo, SharedAttachment, SharedNote,
        SharedProjectData, TempoResult,
    },
    structure::{file_exists, get_file_path},
    tempo::Tempo,
    types::{
        Attachment, AudioAttachment, Comment, FileInfo, FileMeta, NoteDoc, ProjectAttachment,
        ProjectData, RepliableComment,
    },
};

#[derive(Debug)]
pub struct Note {
    tempo: Tempo,
    // emitter: StateEmitter,
    folder: FolderInner,
    channel: ChannelInner,

    note_ulid: String,
    doc: NoteDoc,
}

impl Note {
    pub fn load(
        tempo: Tempo,
        folder: FolderInner,
        channel: ChannelInner,
        note_ulid: &str,
        // emitter: StateEmitter,
    ) -> Result<Self> {
        let doc = NoteDoc::load(
            &folder.path()?,
            &folder.username()?,
            channel.ulid().as_deref(),
            note_ulid,
        )?;

        Ok(Self {
            tempo,
            folder,
            channel,
            note_ulid: note_ulid.to_owned(),
            doc,
        })
    }

    pub fn create(
        tempo: Tempo,
        folder: FolderInner,
        channel: ChannelInner,
        note: NewNote,
    ) -> Result<Self> {
        let (note_ulid, doc) = NoteDoc::create(
            &folder.path()?,
            &folder.username()?,
            channel.ulid().as_deref(),
            note,
        )?;

        Ok(Self {
            tempo,
            folder,
            channel,
            note_ulid: note_ulid.to_owned(),
            doc,
        })
    }

    pub fn add_comment(mut self, comment: NewComment) -> Result<Self> {
        if comment.body.is_empty() {
            return Err(TempoError::Note(
                "Comments cannot have an empty body".into(),
            ));
        }

        if self.doc.attachment.is_none() {
            return Err(TempoError::Note(
                "Cannot reply to a note without an attachment".into(),
            ));
        }

        let new_comment = Comment {
            sender: self.folder.username()?,
            body: comment.body,
        };

        if let Some(reply_ulid) = comment.reply_ulid {
            self.doc
                .comments
                .get_mut(&reply_ulid)
                .ok_or(TempoError::Note(
                    "Attempted to reply to an unknown comment".into(),
                ))?
                .replies
                .insert(new_ulid(), new_comment);
        } else {
            self.doc.comments.insert(
                new_ulid(),
                RepliableComment {
                    comment: new_comment,
                    replies: HashMap::new(),
                },
            );
        }

        let (_new_hash, new_doc) = self.doc.save(
            &self.folder.path()?,
            &self.folder.username()?,
            self.channel.ulid().as_deref(),
            &self.note_ulid,
        )?;

        Ok(Self {
            tempo: self.tempo,
            folder: self.folder,
            channel: self.channel,
            note_ulid: self.note_ulid,
            doc: new_doc,
        })
    }

    pub fn attachment(&self) -> Option<&Attachment> {
        self.doc.attachment.as_ref()
    }

    pub fn get(&self) -> Result<TempoResult<SharedNote>> {
        Ok(SharedNote::new(
            &self.folder.path()?,
            &self.folder.username()?,
            self.channel.ulid().as_deref(),
            &self.note_ulid,
            &self.tempo.get_data_dir_db()?.ok_or(TempoError::Db(
                "Please scan your plugins. Missing plugin database".into(),
            ))?,
        ))
    }

    // for testing
    #[allow(dead_code)]
    pub fn doc(&self) -> &NoteDoc {
        &self.doc
    }

    #[allow(dead_code)]
    pub fn ulid(&self) -> &str {
        &self.note_ulid
    }
}

impl NewAttachment {
    pub fn create(self, folder: &Path, username: &str) -> Result<Attachment> {
        Ok(match self {
            NewAttachment::Project(NewProjectAttachment {
                title,
                path,
                render,
            }) => Attachment::Project(ProjectAttachment {
                title,
                hash: add_file(folder, username, &path)?,
                render_hash: render.map(|r| add_file(folder, username, &r)).transpose()?,
            }),
            NewAttachment::Audio(NewAudioAttachment { title, path }) => {
                Attachment::Audio(AudioAttachment {
                    title,
                    hash: add_file(folder, username, &path)?,
                })
            }
        })
    }
}

impl SharedNote {
    pub fn new(
        folder: &Path,
        username: &str,
        channel_ulid: Option<&str>,
        note_ulid: &str,
        db: &SharedDb,
    ) -> TempoResult<Self> {
        let doc = match NoteDoc::load(folder, username, channel_ulid, note_ulid) {
            Ok(d) => d,
            Err(e) => return TempoResult::Err(format!("Failed to load note doc: {e}")),
        };

        TempoResult::Ok(Self {
            sender: doc.sender,
            body: doc.body,
            reply_ulid: doc.reply_ulid,
            attachment: doc.attachment.map(|a| SharedAttachment::new(folder, a, db)),
            comments: doc.comments,
        })
    }
}

impl ProjectInfo {
    pub fn new(folder: &Path, hash: &str, db: &SharedDb) -> TempoResult<ProjectInfo> {
        match file_exists(folder, hash) {
            Err(e) => {
                return TempoResult::Err(format!(
                    "Failed to check if project file copy exists: {e}"
                ))
            }
            Ok(false) => {
                return TempoResult::Err(format!(
                    "Failed to find project file, it might still be syncing: {}",
                    path_to_str(&get_file_path(folder, hash))
                ))
            }
            Ok(true) => (),
        }

        let file_info = match FileInfo::load(folder, hash) {
            Ok(i) => i,
            Err(e) => {
                return TempoResult::Err(format!("Failed to load project file metadata: {e}"))
            }
        };
        let data = match file_info.meta {
            FileMeta::Project(data) => SharedProjectData::new(folder, data, db),
            o => {
                return TempoResult::Err(format!(
                    "Corrupt file metadata: expected Project, found {:#?}",
                    o
                ))
            }
        };
        TempoResult::Ok(ProjectInfo {
            filename: file_info.filename,
            data,
        })
    }
}

impl SharedProjectData {
    /// Scans to see if database contains all plugins.
    /// Checks for presence of all referenced files.
    pub fn new(folder: &Path, data: ProjectData, db: &SharedDb) -> Self {
        match data {
            ProjectData::Ableton { refs, plugins } => {
                let mut missing_files = vec![];
                for (hash, filename) in refs {
                    let exists = match file_exists(folder, &hash) {
                        Ok(e) => e,
                        Err(e) => {
                            error!("SharedProjectData::new(): error while trying to read file {hash}, treating as missing: {e}");
                            false
                        }
                    };
                    if !exists {
                        missing_files.push(filename)
                    }
                }

                let mut missing_plugins = vec![];
                for plugin in plugins {
                    match db.get_ableton_plugin(&plugin) {
                        Ok(None) => missing_plugins.push(PluginRef::from(plugin)),
                        Ok(Some(_)) => (),
                        Err(e) => {
                            error!("SharedProjectData::new(): error while reading plugin: {:#?}, error: {e}, treating as missing", &plugin);
                            missing_plugins.push(PluginRef::from(plugin))
                        }
                    }
                }
                Self::Ableton {
                    missing_files,
                    missing_plugins,
                }
            }
        }
    }
}

impl AudioFileInfo {
    pub fn new(folder: &Path, hash: &str) -> TempoResult<AudioFileInfo> {
        let file_info = match FileInfo::load(folder, hash) {
            Ok(i) => i,
            Err(e) => return TempoResult::Err(format!("Failed to load audio file metadata: {e}")),
        };

        let filename = file_info.filename;

        let exists = match file_exists(folder, hash) {
            Ok(e) => e,
            Err(e) => return TempoResult::from(e),
        };

        if !exists {
            return TempoResult::Err(format!(
                "Missing local copy of {filename}, it might still be syncing"
            ));
        }

        TempoResult::Ok(AudioFileInfo {
            path: get_file_path(folder, hash),
            filename,
        })
    }
}
