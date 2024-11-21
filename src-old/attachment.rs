use std::path::{Path, PathBuf};

use crate::{
    daw::ableton::copy_ableton_project,
    db::SharedDb,
    file::get_unique_dir,
    misc::{Result, TempoError},
    shared::{
        AudioFileInfo, FileErr, ProjectInfo, SharedAttachment, SharedAudioAttachment,
        SharedProjectAttachment,
    },
    types::{Attachment, AudioAttachment, FileInfo, FileMeta, ProjectAttachment, ProjectData},
};

impl ProjectAttachment {
    pub fn copy(&self, folder: &Path, dest_dir: &Path) -> Result<(PathBuf, Vec<FileErr>)> {
        let project_info = load_file_info(folder, &self.hash, "project")?;

        let project_data = match project_info.meta {
            FileMeta::Project(d) => d,
            t => handle_unexpected_filemeta_variant(t, "project")?,
        };

        // the directory that the project file will be copied into
        // aka "live project"
        let project_dir = get_unique_dir(dest_dir, &format!("[tempo] {}", &self.title))?;

        let errs = match project_data {
            ProjectData::Ableton { refs, plugins: _ } => copy_ableton_project(
                folder,
                &self.hash,
                &project_info.filename,
                &refs,
                &project_dir,
            )?,
        };

        Ok((project_dir, errs))
    }

    // pub fn copy_render(&self, folder: &Path, dest_dir: &Path) -> Result<PathBuf> {
    //     todo!()
    // }
}

// impl AudioAttachment {
//     pub fn copy(&self, folder: &Path, dest_dir: &Path) -> Result<PathBuf> {
//         let project_info = load_file_info(folder, &self.hash, "audio")?;

//         match project_info.meta {
//             FileMeta::Audio(_) => (),
//             t => handle_unexpected_filemeta_variant(t, "project")?,
//         }

//         let copy_path = get_unique_filename(dest_dir, &project_info.filename)?;

//         fs::copy(get_file_path(folder, &self.hash), &copy_path)?;

//         Ok(copy_path)
//     }
// }

fn load_file_info(folder: &Path, hash: &str, expected: &str) -> Result<FileInfo> {
    match FileInfo::load(folder, hash) {
        Ok(i) => Ok(i),
        Err(e) => Err(TempoError::Note(format!(
            "Failed to load metadata for attached {expected}: {e}"
        ))),
    }
}

fn handle_unexpected_filemeta_variant<T>(v: FileMeta, expected: &str) -> Result<T> {
    Err(TempoError::Note(format!(
        "Possible corruption: expected {expected} metadata, instead found: {:#?}",
        v
    )))
}

impl SharedAttachment {
    pub fn new(folder: &Path, attachment: Attachment, db: &SharedDb) -> Self {
        match attachment {
            Attachment::Project(ProjectAttachment {
                title,
                hash,
                render_hash,
            }) => Self::Project(SharedProjectAttachment {
                title,
                project: ProjectInfo::new(folder, &hash, db),
                render: render_hash.map(|h| AudioFileInfo::new(folder, &h)),
            }),

            Attachment::Audio(AudioAttachment { title, hash }) => {
                Self::Audio(SharedAudioAttachment {
                    title,
                    file: AudioFileInfo::new(folder, &hash),
                })
            }
        }
    }
}
