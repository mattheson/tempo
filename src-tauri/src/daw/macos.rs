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

use serde::{Deserialize, Serialize};

use crate::db::SharedAudioUnitRow;
use crate::misc::{Result, TempoError};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
pub struct AudioUnitId {
    // Ableton: ComponentType tag
    pub au_type: u32,
    // Ableton: ComponentSubType tag
    pub au_subtype: u32,
    // Ableton: ComponentManufacturer tag
    pub manufacturer: u32,
}

/// Scans all audio units on macOS.
#[cfg(target_os = "macos")]
pub fn scan_audio_units() -> Result<Vec<SharedAudioUnitRow>> {
    use coreaudio_sys::{
        kAudioUnitType_Effect, kAudioUnitType_FormatConverter, kAudioUnitType_Generator,
        kAudioUnitType_MIDIProcessor, kAudioUnitType_Mixer, kAudioUnitType_MusicDevice,
        kAudioUnitType_MusicEffect, kAudioUnitType_OfflineEffect, kAudioUnitType_Output,
        kAudioUnitType_Panner,
    };
    use log::error;

    // macos has various types of audio components, we want to focus on audio units
    // componentType can be checked to see the type of the audio component, there are various enums we compare against
    const TYPES: &[u32] = &[
        kAudioUnitType_Output,
        kAudioUnitType_MusicDevice,
        kAudioUnitType_MusicEffect,
        kAudioUnitType_FormatConverter,
        kAudioUnitType_Effect,
        kAudioUnitType_Mixer,
        kAudioUnitType_Panner,
        kAudioUnitType_OfflineEffect,
        kAudioUnitType_Generator,
        kAudioUnitType_MIDIProcessor,
    ];

    unsafe {
        use coreaudio_sys::{
            AudioComponent, AudioComponentCopyName, AudioComponentDescription,
            AudioComponentFindNext, AudioComponentGetDescription, CFStringRef,
        };
        use std::ptr;

        unsafe fn cf_string_to_string(cf_string_ref: CFStringRef) -> Result<String> {
            // this feels kind of sketchy but it's just a pointer
            // also i think core_foundation has drop impl for CF types? so we shouldnt need to free this
            let cf_string: core_foundation::string::CFString =
                core_foundation::base::TCFType::wrap_under_create_rule(
                    cf_string_ref as core_foundation::string::CFStringRef,
                );

            Ok(cf_string.to_string())
        }

        let mut plugins: Vec<SharedAudioUnitRow> = vec![];

        let null_desc = AudioComponentDescription {
            componentType: 0,
            componentSubType: 0,
            componentManufacturer: 0,
            componentFlags: 0,
            componentFlagsMask: 0,
        };

        let mut comp: AudioComponent = ptr::null_mut();

        loop {
            comp = AudioComponentFindNext(comp, &null_desc);
            if comp.is_null() {
                break;
            }

            let mut desc = AudioComponentDescription {
                componentType: 0,
                componentSubType: 0,
                componentManufacturer: 0,
                componentFlags: 0,
                componentFlagsMask: 0,
            };

            // noErr == 0
            match AudioComponentGetDescription(comp, &mut desc) {
                0 => (),
                v => {
                    return Err(TempoError::MacOs(format!(
                        "AudioComponentGetDescription failed with OSStatus {v}"
                    )))
                }
            }

            if !TYPES.contains(&desc.componentType) {
                continue;
            }

            let mut cf_name: CFStringRef = ptr::null_mut();

            match AudioComponentCopyName(comp, &mut cf_name) {
                0 => (),
                v => {
                    return Err(TempoError::Plugin(format!(
                        "AudioComponentCopyName failed with OSStatus {v}",
                    )))
                }
            }

            let vend_name = cf_string_to_string(cf_name)?;

            // this line causes sigterm when plugins are scanned twice
            // probably a double free
            // CFRelease(cf_name as *mut _);

            // println!("{vend_name}: {:#?}", &desc);

            let (vendor, name) = match vend_name.split_once(": ") {
                Some((vendor, name)) => (vendor.trim(), name.trim()),
                None => {
                    error!("failed to parse audio unit plugin name {}", vend_name);
                    continue;
                }
            };

            plugins.push(SharedAudioUnitRow {
                au_type: desc.componentType,
                au_subtype: desc.componentSubType,
                au_manufacturer: desc.componentManufacturer,
                name: name.to_string(),
                vendor: vendor.to_string(),
            });
        }

        Ok(plugins)
    }
}

#[cfg(not(target_os = "macos"))]
pub fn scan_audio_units() -> Result<Vec<SharedAudioUnitRow>> {
    Ok(vec![])
}
