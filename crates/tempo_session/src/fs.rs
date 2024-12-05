
pub struct FsSession {}

impl super::Session for FsSession {
    type Err = FsSessionError;
}

#[derive(thiserror::Error, Debug)]
pub enum FsSessionError {
    #[error("io error")]
    Io(#[from] std::io::Error)
}

/// Object store similar to Git's object store.
struct FsObjectStore {
    dir: std::path::PathBuf,
}

impl FsObjectStore {
    pub fn new<P>(dir: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        if std::fs::exists(dir.as_ref())? {
            if is_valid_object_store(dir.as_ref())? {
                Ok(Self {
                    dir: dir.as_ref().to_path_buf()
                })
            } else {
                Err(anyhow::anyhow!("{} is not a valid object store", dir.as_ref().to_string_lossy()))
            }
        } else {
            std::fs::create_dir_all(dir.as_ref())?;
            Ok(Self {
                dir: dir.as_ref().to_path_buf()
            })
        }
    }
}

fn is_valid_object_store(dir: &std::path::Path) -> anyhow::Result<bool> {
    todo!()
}