pub mod audio;

use std::{
    fs,
    path::{Path, PathBuf},
};

use log::{error, info};

use crate::{
    daw::{ableton::add_ableton_project, project::ProjectType},
    misc::{
        extract_file_extension, get_filename, get_unix_timestamp, hash_file, path_to_str, Result,
        TempoError,
    },
    shared::AttachmentType,
    structure::{expect_valid_folder, get_file_dir_path, get_file_meta_path, get_file_path},
    types::{AudioType, FileInfo, FileMeta, FileType},
};

use audio::{is_ableton_drmed_aif, is_max_for_live_patch};

impl FileType {
    pub fn get(path: &Path) -> Result<Self> {
        if path.is_dir() {
            return Err(TempoError::File(format!(
                "FileType::get() was provided with a directory: {}",
                path_to_str(path)
            )));
        }

        if let Some(project_type) = ProjectType::get(path)? {
            return Ok(FileType::Project(project_type));
        }

        if let Some(audio_type) = AudioType::get(path)? {
            return Ok(FileType::Audio(audio_type));
        }

        if is_ableton_drmed_aif(path)? {
            return Ok(FileType::Audio(AudioType::Other));
        }

        if is_max_for_live_patch(path)? {
            return Ok(FileType::MaxForLive);
        }

        Ok(FileType::Other)
    }
}

impl AttachmentType {
    pub fn get(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Err(TempoError::File(format!(
                "Please attach a file. {} is not a file",
                path_to_str(path)
            )));
        }

        if let Some(project_type) = ProjectType::get(path)? {
            return Ok(Self::Project(project_type));
        }

        if let Some(audio_type) = AudioType::get(path)? {
            return Ok(Self::Audio(audio_type));
        }

        match is_ableton_drmed_aif(path) {
            Ok(true) => return Err(TempoError::File("You tried to attach a DRMed Ableton .aif file. Please render this file to another audio format if you'd like to attach it.".into())),
            Ok(false) => (),
            Err(e) => error!("FileType::get_attachment(): error while checking for ableton drm: {e}")
        }

        Err(TempoError::File(format!(
            "Unknown or unsupported file attachment: {}",
            path_to_str(path)
        )))
    }
}

impl FileInfo {
    /// Loads a `FileInfo` from a Tempo folder.
    pub fn load(folder: &Path, file_sha256: &str) -> Result<Self> {
        expect_valid_folder(folder)?;
        let file_meta_path = get_file_meta_path(folder, file_sha256);
        if !file_meta_path.exists() {
            return Err(TempoError::File(format!(
                "Could not find info for file {file_sha256}"
            )));
        }
        Ok(serde_json::from_reader(fs::File::open(file_meta_path)?)?)
    }
}

/// For when users directly add files to folders.
pub fn add_file(folder: &Path, username: &str, file: &Path) -> Result<String> {
    Ok(match FileType::get(file)? {
        FileType::Project(t) => match t {
            ProjectType::Ableton => add_ableton_project(folder, username, file)?,
        },
        FileType::Audio(audio_type) => {
            add_file_with_meta(folder, username, file, FileMeta::Audio(audio_type))?
        }
        FileType::MaxForLive => add_file_with_meta(folder, username, file, FileMeta::MaxForLive)?,
        FileType::Other => {
            return Err(TempoError::File(
                "Tried to add unknown/unsupported file type".into(),
            ))
        }
    })
}

/// Allows copying of files of unknown types.
pub fn add_referenced_file(folder: &Path, username: &str, file: &Path) -> Result<String> {
    Ok(match FileType::get(file)? {
        FileType::Project(t) => match t {
            ProjectType::Ableton => add_ableton_project(folder, username, file)?,
        },
        FileType::Audio(audio_type) => {
            add_file_with_meta(folder, username, file, FileMeta::Audio(audio_type))?
        }
        FileType::MaxForLive => add_file_with_meta(folder, username, file, FileMeta::MaxForLive)?,
        FileType::Other => add_file_with_meta(folder, username, file, FileMeta::Other)?,
    })
}

/// Adds a file to a shared folder.
/// Returns the hash of the added file.
pub fn add_file_with_filename(
    folder: &Path,
    username: &str,
    file: &Path,
    filename: &str,
    file_meta: FileMeta,
) -> Result<String> {
    info!("adding file {filename}");

    if !file.exists() || !file.is_file() {
        return Err(TempoError::File(format!(
            "{} does not exist or is not a file",
            path_to_str(file)
        )));
    }

    expect_valid_folder(folder)?;

    let file_sha256 = hash_file(file)?;
    let file_dir_path = get_file_dir_path(folder, &file_sha256);
    let file_path = get_file_path(folder, &file_sha256);
    let file_meta_path = get_file_meta_path(folder, &file_sha256);

    if !file_dir_path.exists() {
        fs::create_dir_all(&file_dir_path)?;
        fs::copy(file, &file_path)?;

        save_new_file_info(
            &FileInfo {
                filename: filename.to_string(),
                added_by: username.into(),
                timestamp: get_unix_timestamp()?,
                meta: file_meta,
            },
            &file_meta_path,
        )?;
    }

    Ok(file_sha256)
}

/// Adds a file to a shared folder. Uses the filename of the provided file.
/// Returns the hash of the added file.
pub fn add_file_with_meta(
    folder: &Path,
    username: &str,
    file: &Path,
    file_meta: FileMeta,
) -> Result<String> {
    let filename = get_filename(file)?;
    add_file_with_filename(folder, username, file, &filename, file_meta)
}

fn save_new_file_info(info: &FileInfo, path: &Path) -> Result<()> {
    Ok(serde_json::to_writer_pretty(
        fs::File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?,
        &info,
    )?)
}

/// Get a unique filename for the provided filename in `dir`.
/// Possible to get `Err` if we don't have read/write perms in `dir`
pub fn get_unique_filename(dir: &Path, filename: &str) -> Result<PathBuf> {
    let mut curr = dir.join(filename);

    if !fs::exists(&curr)? {
        Ok(curr)
    } else {
        let (base, ext) = extract_file_extension(filename);

        let mut count = 1usize;
        loop {
            curr = dir.join(format!("{base}-{count}{}", ext.as_deref().unwrap_or("")));

            if !curr.exists() {
                break Ok(curr);
            }

            count += 1;
        }
    }
}

pub fn get_unique_dir(parent_dir: &Path, dir_name: &str) -> Result<PathBuf> {
    let mut curr = parent_dir.join(dir_name);

    if !fs::exists(&curr)? {
        fs::create_dir_all(&curr)?;
        Ok(curr)
    } else {
        let mut count = 1usize;
        loop {
            curr = parent_dir.join(format!("{dir_name}-{count}"));

            if !curr.exists() {
                fs::create_dir_all(&curr)?;
                break Ok(curr);
            }

            count += 1;
        }
    }
}

// will be used for copying attachments

/// Copies a single file out of a Tempo folder.
#[allow(dead_code)]
pub fn copy_file(folder: &Path, hash: &str, dest_dir: &Path, filename: &str) -> Result<PathBuf> {
    let dest = get_unique_filename(dest_dir, filename)?;
    let src = get_file_path(folder, hash);

    if !fs::exists(&src)? {
        return Err(TempoError::File(format!(
            "Local copy of {filename} is missing"
        )));
    }

    fs::copy(&src, &dest)?;

    Ok(dest)
}
