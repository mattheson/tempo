// =====================================================================================
// This file is licensed under either of
// Apache License, Version 2.0 or MIT license, at your option.
// =====================================================================================
// You may obtain a copy of the Apache License, Version 2.0 at
// http://www.apache.org/licenses/LICENSE-2.0
// =====================================================================================
// You may obtain a copy of the MIT License at
// https://opensource.org/licenses/MIT
// =====================================================================================

mod als;
mod db;
mod project;

pub use als::{verify_project, ProjectFileRefReader, ProjectFileRefWriter, ProjectPluginReader};
pub use db::{have_plugin_db, scan_plugin_db, ScannedAbletonPlugin};
pub use project::{
    add_ableton_project, copy_ableton_project, scan_filerefs, AbletonProjectPluginScan,
};

use serde::{Deserialize, Serialize};

use crate::shared::PluginRef;

use super::{macos::AudioUnitId, plugin::PluginType};

/// Represents a plugin found in an Ableton project file
#[derive(Debug, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum AbletonPluginRef {
    // comments above fields are corresponding names of tags

    // ableton seems to be able to handle if device names are missing
    // im assuming that it requires the device ids though

    // VstPluginInfo
    Vst {
        // UniqueId
        id: u32,
        // PlugName
        name: Option<String>,
    },

    // Vst3PluginInfo
    Vst3 {
        // Fields inside of Uid, for some reason these are signed. maybe to reduce number of chars
        fields: [i32; 4],
        // Name
        name: Option<String>,
    },

    // AuPluginInfo
    Au {
        id: AudioUnitId,
        // Name
        name: Option<String>,
        // Manufacturer
        manufacturer: Option<String>,
    },
}

impl From<AbletonPluginRef> for PluginRef {
    fn from(value: AbletonPluginRef) -> Self {
        let (plugin_type, name, vendor) = match value {
            AbletonPluginRef::Vst { id: _, name } => (PluginType::Vst, name, None),
            AbletonPluginRef::Vst3 { fields: _, name } => (PluginType::Vst3, name, None),
            AbletonPluginRef::Au {
                id: _,
                name,
                manufacturer,
            } => (PluginType::Au, name, manufacturer),
        };

        Self {
            plugin_type,
            name: name.unwrap_or("Unknown name".into()),
            vendor: vendor.unwrap_or("Unknown vendor".into()),
        }
    }
}
