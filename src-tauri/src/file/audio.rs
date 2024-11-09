use std::path::Path;

use log::warn;

use crate::{
    misc::{extract_file_extension, get_filename, path_to_str, Result, TempoError},
    types::AudioType,
};

impl AudioType {
    pub fn get(path: &Path) -> Result<Option<Self>> {
        let mime = infer::get_from_path(path)
            .map_err(|e| TempoError::File(format!("Failed to read file: {e}")))?;

        match mime {
            None => {
                warn!(
                    "get_audio_type(): could not find mime of {}",
                    path_to_str(path)
                );
                Ok(None)
            }
            Some(t) => {
                if t.mime_type().starts_with("audio") {
                    Ok(Some(AudioType::Other))
                } else {
                    warn!(
                        "get_audio_type(): found non-audio {t} for {}",
                        path_to_str(path)
                    );
                    Ok(None)
                }
            }
        }
    }
}

pub fn is_ableton_drmed_aif(path: &Path) -> Result<bool> {
    let (_, ext) = extract_file_extension(&get_filename(path)?);

    if let Some(ext) = ext {
        if ext.as_str() == "aif" {
            // TODO actually read header
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn is_max_for_live_patch(path: &Path) -> Result<bool> {
    let (_, ext) = extract_file_extension(&get_filename(path)?);

    if let Some(ext) = ext {
        if ext.as_str() == "amxd" {
            return Ok(true);
        }
    }

    Ok(false)
}

// pub fn copy_audio_file(folder: &Path, dest: &Path, hash: &str) -> Result<(PathBuf, FileErr)> {
//     match FileInfo::load(folder, hash) {
//         Ok(i) => match i.meta {
//             FileMeta::Audio(_) => {

//             },
//             t => {
//                 Err(TempoError::File("Failed to copy audio file, found non-audio metadata: "))
//             }
//         },
//         Err(e) => Err(TempoError::File(format!("Failed to load file metadata for audio file: {e}")))
//     }
// }
