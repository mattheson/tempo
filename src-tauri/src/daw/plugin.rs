use std::path::Path;

use super::{ableton::{self, AbletonProjectPluginScan}, project::ProjectType};
use autosurgeon::{Hydrate, Reconcile};
use serde::{Deserialize, Serialize};

use crate::{
    misc::{path_to_str, Result, TempoError},
    shared::ProjectFileRefScan,
};

#[derive(
    Eq, PartialEq, Hash, Debug, Clone, Reconcile, Hydrate, Serialize, Deserialize, ts_rs::TS,
)]
pub enum PluginType {
    Au,
    Vst,
    Vst3,
}

pub enum ProjectPluginScan {
    Ableton(AbletonProjectPluginScan),
}

impl ProjectPluginScan {
    pub fn new(path: &Path) -> Result<Self> {
        match ProjectType::get(path)? {
            None => Err(TempoError::Project(format!(
                "Attempted to scan plugins for {} which appears not to be a project file",
                path_to_str(path)
            ))),
            Some(t) => match t {
                ProjectType::Ableton => Ok(Self::Ableton(AbletonProjectPluginScan::new(path)?)),
            },
        }
    }
}

impl ProjectFileRefScan {
    pub fn new(path: &Path) -> Result<Self> {
        match ProjectType::get(path)? {
            None => Err(TempoError::Project(format!(
                "Attempted to scan file refs for {} which appears to not be a project file",
                path_to_str(path)
            ))),
            Some(t) => match t {
                ProjectType::Ableton => Ok(ableton::scan_filerefs(path)?),
            },
        }
    }
}
