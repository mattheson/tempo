use crate::{Error, Result};
use tempo_misc::Key;
use tempo_provider::Value;

pub struct FsMap(std::path::PathBuf);
pub struct FsData(std::path::PathBuf);

impl<'map> tempo_provider::Map<'map> for crate::FsMap {
    type Err = Error;

    fn size(&self) -> Result<usize> {
        Ok(read_dir_keys(&self.0)?.count())
    }

    fn iter(
            &'map self,
        ) -> Result<impl Iterator<Item = (Key, Value<'map, FsMap, FsData>)>> {
        
    }

    fn exists(&self, key: &Key) -> Result<bool> {
        todo!()
    }

    fn get(&'map self, path: &Key) -> Result<Option<Value<FsMap<'map>, FsData>>> {
        todo!()
    }

    fn set(&self, key: &Key, data: impl std::io::Read) -> Result<()> {
        todo!()
    }

    fn clear(&self, _key: &Key) -> Result<()> {
        panic!("clear not implemented for FsSession map")
    }
}

impl<'data> tempo_provider::Data<'data> for FsData<'data> {
    type Err = Error;

    fn read(&self) -> Result<impl std::io::Read> {
        Ok(ReadPath::new(self.build_path()?)?)
    }
}

/// Wrapper around a `PathBuf`.
pub struct ReadPath {
    path: std::path::PathBuf,
    file: std::fs::File,
}

impl ReadPath {
    pub fn new(path: std::path::PathBuf) -> std::io::Result<Self> {
        let file = std::fs::File::open(&path)?;
        Ok(ReadPath { path, file })
    }
}

impl std::io::Read for ReadPath {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

/// `std::fs::read_dir()` which filters out invalid entries, i.e. those which have an invalid `Key`.
fn read_dir_keys(
    path: &std::path::Path,
) -> Result<impl Iterator<Item = Key>> {
    let path_lossy = path.to_string_lossy().to_string();

    Ok(std::fs::read_dir(path)?.filter_map(move |ent| {
        match ent {
            Err(err) => {
                log::warn!("filtering out Err in {}: {}", path_lossy, err);
                None
            }
            Ok(ent) => {
                match ent.file_name().try_into() {
                    Ok(key) => Some(key),
                    Err(err) => {
                        log::warn!("could not convert into valid Key in {}, filtering: {}", path_lossy, err);
                        None
                    }
                }
            }
        }
    }))
}
