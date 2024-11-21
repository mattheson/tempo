use std::path::Path;

use crate::{
    misc::{Result, TempoError},
    shared::{NewAttachment, NewAudioAttachment, NewNote, NewProjectAttachment},
    structure::note_exists,
    types::FileType,
};

// checking the fields of user-created data types

// for example we don't want users to create notes with project attachments with no title
// we verify that the title is not empty
// TODO actually maybe we do want this

pub trait Verifiable {
    /// Verifies the fields of this type.
    fn verify(&self) -> Result<()>;
}

pub trait VerifiableWithInfo {
    /// Verifies the fields of this type.
    fn verify(&self, folder: &Path, channel_ulid: Option<&str>) -> Result<()>;
}

impl VerifiableWithInfo for NewNote {
    fn verify(&self, folder: &Path, channel_ulid: Option<&str>) -> Result<()> {
        if self.body.is_empty() && self.attachment.is_none() {
            return Err(TempoError::Note(
                "Notes can only have an empty body with an attachment".into(),
            ));
        }
        if let Some(r) = self.reply_ulid.as_ref() {
            if !note_exists(folder, channel_ulid, r)? {
                return Err(TempoError::Note("Tried to reply to unknown note".into()));
            }
        }
        if let Some(a) = self.attachment.as_ref() {
            a.verify()?;
        }
        Ok(())
    }
}

impl Verifiable for NewAttachment {
    fn verify(&self) -> Result<()> {
        match self {
            NewAttachment::Project(NewProjectAttachment {
                title,
                path,
                render,
            }) => {
                if title.is_empty() {
                    return Err(TempoError::Note(
                        "A project cannot have an empty title".into(),
                    ));
                }
                match FileType::get(path)? {
                    FileType::Project(_) => (),
                    t => {
                        return Err(TempoError::Note(format!(
                            "Expected a project file as an attachment, found {:#?} instead",
                            t
                        )))
                    }
                }
                if let Some(render) = render.as_ref() {
                    match FileType::get(render)? {
                        FileType::Audio(_) => (),
                        t => {
                            return Err(TempoError::Note(format!(
                                "Expected an audio file as a render, found {:#?} instead",
                                t
                            )))
                        }
                    }
                }
            }
            NewAttachment::Audio(NewAudioAttachment { title, path }) => {
                if let Some(title) = title.as_ref() {
                    if title.is_empty() {
                        return Err(TempoError::Note(
                            "An audio attachment cannot have an empty string as a title".into(),
                        ));
                    }
                }
                match FileType::get(path)? {
                    FileType::Audio(_) => (),
                    t => {
                        return Err(TempoError::Note(format!(
                            "Expected an audio file as an attachment, found {:#?} instead",
                            t
                        )))
                    }
                }
            }
        }
        Ok(())
    }
}
