pub struct UniqueDir {
    pub name: String,
    pub path: std::path::PathBuf,
}

impl UniqueDir {
    pub fn new(dir: impl AsRef<std::path::Path>, name: &str) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir.as_ref())?;

        let create = |p: &std::path::Path| match std::fs::create_dir(p) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(e),
        };

        let mut path = dir.as_ref().join(name);

        if create(&path)? {
            Ok(Self {
                name: name.to_string(),
                path,
            })
        } else {
            let mut count = 1usize;
            loop {
                let curr_name = format!("{name}-{count}");
                path = dir.as_ref().join(&curr_name);

                if create(&path)? {
                    break Ok(Self {
                        name: curr_name,
                        path,
                    });
                }

                count += 1;
            }
        }
    }
}

/// A unique file within a directory.
pub struct UniqueFile {
    pub filename: String,
    pub path: std::path::PathBuf,
    pub file: std::fs::File,
}

impl UniqueFile {
    pub fn new(dir: impl AsRef<std::path::Path>, filename: &str) -> std::io::Result<Self> {
        let mut path = dir.as_ref().join(filename);

        if !std::fs::exists(&path)? {
            Ok(Self {
                filename: filename.to_string(),
                file: std::fs::File::create_new(&path)?,
                path,
            })
        } else {
            let (base, ext) = crate::extract_file_extension(filename);

            let mut count = 1usize;
            loop {
                let curr_filename = format!("{base}-{count}{}", ext.as_deref().unwrap_or(""));
                path = dir.as_ref().join(&curr_filename);

                if !path.exists() {
                    break Ok(Self {
                        filename: curr_filename,
                        file: std::fs::File::create_new(&path)?,
                        path,
                    });
                }

                count += 1;
            }
        }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn file(&mut self) -> &mut std::fs::File {
        &mut self.file
    }
}

pub struct TempDir {
    path: std::path::PathBuf,
    persist: bool,
    save_on_panic: bool,
    panic_if_err: bool,
}

impl TempDir {
    /// Gets a new temporary directory.
    pub fn new(parent_dir: impl AsRef<std::path::Path>, prefix: &str) -> std::io::Result<Self>
    {
        let UniqueDir { name: _, path } = UniqueDir::new(parent_dir, prefix)?;

        log::info!("Created a temporary directory: {}", path.to_string_lossy());

        Ok(Self {
            path,
            persist: false,
            save_on_panic: false,
            panic_if_err: true,
        })
    }

    pub fn persist(mut self) -> Self {
        self.persist = true;
        self
    }

    /// Does not delete this temporary directory when a panic occurs.
    pub fn save_on_panic(mut self) -> Self {
        self.save_on_panic = true;
        self
    }

    /// Panics if an error is encountered when deleting this temporary directory when `Drop`ping.
    pub fn panic_if_err(mut self) -> Self {
        self.panic_if_err = true;
        self
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !((std::thread::panicking() && self.save_on_panic) || self.persist) {
            match std::fs::remove_dir_all(&self.path) {
                Ok(()) => log::info!(
                    "Removed temporary directory at {}",
                    self.path.to_string_lossy()
                ),
                Err(e) => {
                    if self.panic_if_err {
                        panic!("Error while deleting temporary directory: {}", e)
                    } else {
                        log::error!("Error while deleting temporary directory: {}", e)
                    }
                }
            }
        }
    }
}

/// A temporary file with a random location/filename.
/// Automatically attempts to delete the file when `Drop`ped.
pub struct TempFile {
    path: std::path::PathBuf,
    file: std::fs::File,
    persist: bool,
    save_on_panic: bool,
    panic_if_err: bool,
}

impl TempFile {
    pub fn new() -> std::io::Result<Self> {
        // we just name with uuid for now
        let path = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

        let file = std::fs::File::options()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)?;

        Ok(Self {
            file,
            path,
            persist: false,
            save_on_panic: false,
            panic_if_err: false,
        })
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    pub fn persist(mut self) -> Self {
        self.persist = true;
        self
    }

    /// Does not delete this temporary file when a panic occurs.
    pub fn save_on_panic(mut self) -> Self {
        self.save_on_panic = true;
        self
    }

    /// Panics if an error is encountered when deleting this temporary file when `Drop`ping.
    pub fn panic_if_err(mut self) -> Self {
        self.panic_if_err = true;
        self
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !((std::thread::panicking() && self.save_on_panic) || self.persist) {
            match std::fs::remove_file(&self.path) {
                Ok(()) => log::info!("Removed temporary file at {}", self.path.to_string_lossy()),
                Err(e) => {
                    if self.panic_if_err {
                        panic!("Error while deleting temporary file: {}", e)
                    } else {
                        log::error!("Error while deleting temporary file: {}", e)
                    }
                }
            }
        }
    }
}

impl std::io::Read for TempFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl std::io::Write for TempFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Blake3(pub(crate) String);

impl Blake3 {
    pub fn new(hash: &str) -> Result<Self, String> {
        if hash.len() != 64 {
            return Err(format!("invalid length: expected 64, got {}", hash.len()));
        }

        if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("hash contains non-hexadecimal characters".to_string());
        }

        Ok(Self(hash.to_string()))
    }
}

impl std::str::FromStr for Blake3 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl std::fmt::Display for Blake3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Blake3 {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for Blake3 {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Blake3 {
    pub fn into_inner(self) -> String {
        self.0
    }
}
