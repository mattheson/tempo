use crate::{db::TempoDb, providers::fs::FsProvider};

pub struct Tempo {

    fs: FsProvider
}

impl Tempo {
    pub fn new(data_dir: &Path) -> Result<Self> {

    }
}
