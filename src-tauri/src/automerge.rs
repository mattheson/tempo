// saving and loading of automerge doc types

use crate::{
    misc::{is_sha256, new_ulid, path_to_str, Result, TempoError},
    shared::NewNote,
    structure::{expect_valid_folder, get_channel_meta_path, get_note_path, note_exists},
    types::{ChannelDoc, NoteDoc},
    verify::{Verifiable, VerifiableWithInfo},
};
use automerge::{ActorId, Automerge};
use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile};
use log::{error, warn};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::Path,
};

/// Returns a Vec of the sha256 docs stored in a doc directory.
/// A doc directory is the directory which holds the sha256-named automerge docs.
/// Filters out invalid files/entries.
fn get_doc_entries(dir: &Path) -> Result<Vec<DirEntry>> {
    let entries: Vec<DirEntry> = fs::read_dir(dir)?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                if !is_sha256(&entry.file_name().to_string_lossy()) {
                    warn!(
                        "file not named with hash in doc folder {}",
                        path_to_str(&entry.path())
                    );
                    None
                } else {
                    Some(entry)
                }
            }
            Err(e) => {
                warn!(
                    "error while trying to read doc files in {}, {e}",
                    path_to_str(dir),
                );
                None
            }
        })
        .filter(|entry| match entry.file_type() {
            Err(e) => {
                warn!(
                    "error while trying to read doc in {}, {e}",
                    path_to_str(dir),
                );
                false
            }
            Ok(ft) if ft.is_file() => true,
            Ok(_) => {
                warn!(
                    "found non-file {} while reading doc in {}",
                    entry.file_name().to_string_lossy(),
                    path_to_str(dir)
                );
                false
            }
        })
        .collect();

    if entries.is_empty() {
        return Err(TempoError::Doc(format!(
            "no docs found in dir {}",
            dir.display()
        )));
    }

    Ok(entries)
}

/// Gets an Automerge doc from a doc directory. Does not saved the returned doc to disk.
/// The `Vec<DirEntry>` contains a set of existing documents used to build the returned document.
/// All the docs will be loaded, and will be merged together to build the returned doc.
/// Once the returned document is saved back to disk, these existing documents could be deleted.
fn get_doc_with_prev(dir: &Path, actor_id: &str) -> Result<(Automerge, Vec<DirEntry>)> {
    let entries = get_doc_entries(dir)?;

    let first_doc_path = entries[0].path();
    if !first_doc_path.exists() {
        return get_doc_with_prev(dir, actor_id);
    }

    // TODO
    // there are sync edge cases where loading an automerge doc somehow results in a completely empty doc saved to disk
    // i'm not completely sure what causes this yet, but i'm pretty confident somewhere in the mix an empty byte array is being loaded
    // might be fixed
    let check_empty = |b: Vec<u8>| {
        if b.is_empty() {
            return Err(TempoError::Doc(
                "Loaded an automerge document which contains no bytes. This should not happen!!!"
                    .into(),
            ));
        }
        Ok(b)
    };

    let mut doc = Automerge::load(&check_empty(fs::read(first_doc_path)?)?)?;
    doc.set_actor(ActorId::from(actor_id.as_bytes()));

    for e in entries[1..].iter() {
        let path = e.path();
        if !path.exists() {
            return get_doc_with_prev(dir, actor_id);
        } else {
            let mut other = Automerge::load(&check_empty(fs::read(path)?)?)?;
            doc.merge(&mut other)?;
        }
    }

    Ok((doc, entries))
}

/// Gets the latest version of a doc from a doc directory.
/// If multiple docs are found, the docs will be merged together, saved back to disk, and the saved doc will be returned.
/// Returns hash of returned doc.
fn get_doc(dir: &Path, actor_id: &str) -> Result<(String, Automerge)> {
    let (d, prev) = get_doc_with_prev(dir, actor_id)?;
    let hash = save_doc_with_prev(dir, &d, prev)?;
    Ok((hash, d))
}

// /// Gets the latest Automerge document hash from a document directory.
// /// If one doc named with its hash is found, this will return the hash.
// /// If multiple docs are found, the docs will be merged together, saved back to disk, and the hash of the saved doc will be returned.
// /// Afterwards, the old docs will be deleted.
// fn get_doc_hash(dir: &Path, actor_id: &str) -> Result<String> {
//     let entries = get_doc_entries(dir)?;

//     if entries.len() == 1 {
//         // is this ok?
//         Ok(entries[0].file_name().to_string_lossy().to_string())
//     } else {
//         let (d, prev) = get_doc_with_prev(dir, actor_id)?;
//         Ok(save_doc_with_prev(dir, &d, prev)?)
//     }
// }

/// Saves an automerge doc to the given directory.
/// Returns the sha256 of the saved doc.
fn save_doc(dir: &Path, doc: &Automerge) -> Result<String> {
    let saved = doc.save();
    let mut hasher = Sha256::new();

    hasher.update(&saved);

    let hash = format!("{:x}", hasher.finalize());

    let file_path = dir.join(&hash);

    if !file_path.exists() {
        fs::write(&file_path, saved)?;
    }

    Ok(hash)
}

/// Handles saving a doc to a doc directory with previous versions of the doc.
/// Tries to delete old versions of the docs after saving.
/// Returns hash of the new, saved doc.
fn save_doc_with_prev(dir: &Path, doc: &Automerge, prev: Vec<DirEntry>) -> Result<String> {
    let hash = save_doc(dir, doc)?;

    // delete old docs that were merged into doc
    for entry in prev {
        let prev_path = entry.path();
        let prev_filename = entry.file_name();
        let prev_hash = prev_filename.to_string_lossy();

        if prev_hash != hash && prev_path.exists() {
            if let Err(e) = fs::remove_file(entry.path()) {
                error!(
                    "failed to delete old doc {}: {}",
                    entry.file_name().to_string_lossy(),
                    e
                );
            }
        }
    }

    Ok(hash)
}

/// Saves a new doc struct to a given directory.
/// Returns hash of saved doc.
fn save_new_doc<D>(doc: &D, dir: &Path, actor_id: &str) -> Result<String>
where
    D: Reconcile,
{
    fs::create_dir_all(dir)?;

    let mut am_doc = Automerge::new();
    am_doc.set_actor(ActorId::from(actor_id.as_bytes()));

    let mut tx = am_doc.transaction();
    reconcile(&mut tx, doc)?;
    tx.commit();

    save_doc(dir, &am_doc)
}

/// Saves an edited `Doc` struct back to disk.
/// Returns hash of newly saved doc.
fn save_edited_doc<D>(doc: &D, dir: &Path, actor_id: &str) -> Result<String>
where
    D: Hydrate + Reconcile,
{
    let (mut disk_doc, prev) = get_doc_with_prev(dir, actor_id)?;

    let mut tx = disk_doc.transaction();
    reconcile(&mut tx, doc)?;
    tx.commit();

    save_doc_with_prev(dir, &disk_doc, prev)
}

impl ChannelDoc {
    /// Creates a new `ChannelDoc` in the given Tempo folder.
    /// Returns ulid and doc.
    pub fn create(folder: &Path, actor_id: &str, channel_name: &str) -> Result<(String, Self)> {
        expect_valid_folder(folder)?;

        let ulid = new_ulid();

        let meta_dir = get_channel_meta_path(folder, &ulid);

        let doc = ChannelDoc {
            name: channel_name.into(),
            creator: actor_id.into(),
            hidden: false,
        };

        save_new_doc(&doc, &meta_dir, actor_id)?;

        Ok((ulid, doc))
    }

    /// Loads a ChannelDoc.
    /// Note that the global channel has no ChannelDoc.
    pub fn load(folder: &Path, actor_id: &str, channel_ulid: &str) -> Result<Self> {
        expect_valid_folder(folder)?;

        let meta_dir = get_channel_meta_path(folder, channel_ulid);

        if !meta_dir.exists() {
            return Err(TempoError::Channel(format!(
                "unknown channel {channel_ulid} in {}",
                path_to_str(folder)
            )));
        }

        let (_, doc) = get_doc(&meta_dir, actor_id)?;

        let d: ChannelDoc = hydrate(&doc)?;

        Ok(d)
    }

    // /// Saves any changes made to this `ChannelDoc` back to disk.
    // /// Returns hash of saved doc.
    // pub fn save(self, folder: &Path, username: &str, channel_ulid: &str) -> Result<(String, Self)> {
    //     // this takes ownership of self since it's possible that saving the doc could fail
    //     // if this happens we need to discard the changes and try to rebuild the doc again from disk
    //     self.verify()?;
    //     Ok((
    //         save_edited_doc(
    //             &self,
    //             &get_channel_meta_path(folder, channel_ulid),
    //             username,
    //         )?,
    //         self,
    //     ))
    // }
}

impl Verifiable for ChannelDoc {
    fn verify(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(TempoError::Channel(
                "Channels cannot have empty names".into(),
            ));
        }
        Ok(())
    }
}

impl NoteDoc {
    /// Creates and saves a `NoteDoc` to disk.
    /// Returns ulid and doc.
    pub fn create(
        folder: &Path,
        username: &str,
        channel_ulid: Option<&str>,
        note: NewNote,
    ) -> Result<(String, Self)> {
        expect_valid_folder(folder)?;
        note.verify(folder, channel_ulid)?;

        let note_ulid = new_ulid();
        let note_path = get_note_path(folder, channel_ulid, &note_ulid);

        let doc = Self {
            sender: username.to_owned(),
            body: note.body,
            reply_ulid: note.reply_ulid,
            attachment: note
                .attachment
                .map(|a| a.create(folder, username))
                .transpose()?,
            comments: HashMap::new(),
        };

        save_new_doc(&doc, &note_path, username)?;

        Ok((note_ulid, doc))
    }

    pub fn load(
        folder: &Path,
        actor_id: &str,
        channel_ulid: Option<&str>,
        note_ulid: &str,
    ) -> Result<Self> {
        expect_valid_folder(folder)?;

        if !note_exists(folder, channel_ulid, note_ulid)? {
            return Err(TempoError::Note(format!(
                "Unknown note {note_ulid} in channel {} in {}",
                channel_ulid.unwrap_or("global"),
                path_to_str(folder)
            )));
        }

        let (_, doc) = get_doc(&get_note_path(folder, channel_ulid, note_ulid), actor_id)?;

        let d: NoteDoc = hydrate(&doc)?;

        Ok(d)
    }

    /// Saves any changes made to this `NoteDoc` back to disk.
    /// Returns hash of saved doc.
    pub fn save(
        self,
        folder: &Path,
        username: &str,
        channel_ulid: Option<&str>,
        note_ulid: &str,
    ) -> Result<(String, Self)> {
        Ok((
            save_edited_doc(
                &self,
                &get_note_path(folder, channel_ulid, note_ulid),
                username,
            )?,
            self,
        ))
    }
}
