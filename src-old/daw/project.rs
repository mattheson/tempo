use std::path::Path;

use autosurgeon::{Hydrate, Reconcile};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::misc::{extract_file_extension, get_filename, Result, TempoError};

use super::ableton;

#[derive(Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, Eq, PartialEq, ts_rs::TS)]
pub enum ProjectType {
    Ableton,
}

impl ProjectType {
    pub fn get(path: &Path) -> Result<Option<Self>> {
        let full_filename = get_filename(path)?;
        let (filename, ext) = extract_file_extension(&full_filename);

        match ext.as_deref() {
            Some("als") => {
                if let Err(e) = ableton::verify_project(path) {
                    Err(TempoError::Ableton(format!("An error occurred while verifying whether {full_filename} is a valid Ableton project: {e}")))
                } else {
                    Ok(Some(ProjectType::Ableton))
                }
            }
            None => {
                warn!("get_project_type(): scanned {filename}, appears to have no filename");
                Ok(None)
            }
            Some(e) => {
                warn!("get_project_type(): scanned {filename}.{e}, {e} is not a known project file type");
                Ok(None)
            }
        }
    }
}
