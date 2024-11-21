// use std::sync::Arc;

// use tauri::{AppHandle, Emitter};

// use crate::{shared::{FolderInfo, FolderData}, misc::Result};

/*
possible stuff for better state sync in the future

/// Monitors state of Tempo folders for reactive updates.
/// Emits events containing folder state.
#[derive(Clone)]
pub struct StateEmitter {
    handle: Arc<AppHandle>,
}

// emits FolderEmits
pub const FOLDER_EMIT: &str = "tempo://folder";

// emits StateEmits
pub const STATE_EMIT: &str = "tempo://state";

impl StateEmitter {
    pub fn new(handle: Arc<AppHandle>) -> Self {
        Self { handle }
    }

    pub fn emit_folder(&self, folder: &FolderInfo) -> Result<()> {
        Ok(self.handle.emit(FOLDER_EMIT, folder)?)
    }

    pub fn emit_state(&self, state: &FolderData) -> Result<()> {
        Ok(self.handle.emit(STATE_EMIT, state)?)
    }
}
*/

// TODO scanning of project files and certain operations can be pretty slow, would be nice to have progress bar system

/*
static PROGRESS_IDS: LazyLock<Arc<Mutex<HashSet<String>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashSet::new())));

pub struct ProgressEmitter {
    handle: Arc<AppHandle>,
    id: String,
}

fn gen_progress_id() -> String {
    let mut num: usize = 0;
    loop {
        let id = format!("tempo://progress-{num}");
        if PROGRESS_IDS.lock().unwrap().contains(&id) {
            num += 1;
        } else {
            break id;
        }
    }
}

impl ProgressEmitter {
    pub fn new(handle: Arc<AppHandle>) -> Self {
        Self {
            handle,
            id: gen_progress_id(),
        }
    }
    // pub fn emit_progress(progress: u32)
}

impl Drop for ProgressEmitter {
    fn drop(&mut self) {
        let _ = PROGRESS_IDS.lock().unwrap().remove(&self.id);
    }
}
 */
