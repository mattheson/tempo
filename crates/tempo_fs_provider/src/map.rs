use crate::{Error, Result};
use tempo_misc::Key;
use tempo_provider::Value;

pub struct FsMap(std::path::PathBuf);

impl tempo_provider::Map for crate::FsMap {
    type Err = Error;

    type Data = FsData;

    fn size(&self) -> u64 {
        todo!()
    }

    fn iter(&self) -> Result<impl Iterator<Item = (Key, Value<FsMap, FsData>)>> {
        todo!()
    }

    fn exists(&self, path: &[tempo_misc::Key]) -> Result<bool> {
        todo!()
    }

    fn get(&self, path: &[tempo_misc::Key]) -> Result<Option<Value<Self, FsData>>> {
        todo!()
    }

    fn set(&self, path: &[tempo_misc::Key], data: impl std::io::Read) -> Result<()> {
        todo!()
    }

    fn clear(&self, _path: &[tempo_misc::Key]) -> Result<()> {
        panic!("clear not implemented for FsSession map")
    }
}

impl tempo_provider::Map for crate::FsSession {
    type Err = Error;

    type Data = FsData;

    fn size(&self) -> u64 {
        todo!()
    }

    fn iter(&self) -> Result<impl Iterator<Item = (Key, Value<FsMap, FsData>)>> {
        todo!()
    }

    fn exists(&self, path: &[tempo_misc::Key]) -> Result<bool> {
        todo!()
    }

    fn get(&self, path: &[tempo_misc::Key]) -> Result<Option<Value<Self, FsData>>> {
        todo!()
    }

    fn set(&self, path: &[tempo_misc::Key], data: impl std::io::Read) -> Result<()> {
        todo!()
    }

    fn clear(&self, _path: &[tempo_misc::Key]) -> Result<()> {
        panic!("clear not implemented for FsSession map")
    }
}

struct FsData {}

impl tempo_provider::Data for FsData {
    type Err = Error;

    fn new(data: impl std::io::Read) -> Result<Self> {
        todo!()
    }

    fn read(&self) -> Result<impl std::io::Read> {
        todo!()
    }
}
