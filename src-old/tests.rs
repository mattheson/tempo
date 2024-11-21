// really big TODO
// these tests aren't great and it would be good to add e2e testing with tauri somehow

// TODO
// some tests will fail if you don't have Ableton installed and/or do not have full disk access enabled for the parent process on macOS
// (particularly any which involve scanning plugin database)
// would be nice to add stuff in build.rs to check this ahead of time

#[allow(dead_code)]
pub fn get_temp_file(prefix: &str) -> std::path::PathBuf {
    use std::fs;

    #[cfg(target_os = "macos")]
    let temp_dir = std::path::PathBuf::from("/tmp");
    #[cfg(not(target_os = "macos"))]
    let temp_dir = std::env::temp_dir();

    let mut path = temp_dir.join(prefix);

    let expect_exists = |p: &std::path::Path| {
        fs::exists(p)
            .unwrap_or_else(|_| panic!("failed to check if {} exists", p.to_string_lossy()))
    };

    if expect_exists(&path) {
        let mut dup = 1usize;
        loop {
            path = temp_dir.join(format!("{}-{}", prefix, dup));

            if !expect_exists(&path) {
                break path;
            }

            dup += 1;
        }
    } else {
        path
    }
}

#[allow(dead_code)]
fn expect_create_dir_all(p: &std::path::Path) {
    std::fs::create_dir_all(p)
        .unwrap_or_else(|_| panic!("failed to create testing dir {}", p.to_string_lossy()))
}

#[allow(dead_code)]
pub fn get_temp_dir(prefix: &str) -> std::path::PathBuf {
    let p = get_temp_file(prefix);
    expect_create_dir_all(&p);
    p
}

/// Tempo instance for testing
#[allow(dead_code)]
struct Testpo {
    pub tempo: crate::Tempo,
    pub test_dir: std::path::PathBuf,
    pub data_dir: std::path::PathBuf,
}

impl Testpo {
    /// Creates Tempo instance, performs an initial plugin scan
    pub fn new(prefix: &str) -> Self {
        use crate::tempo::Tempo;

        let test_dir = get_temp_dir(prefix);
        let data_dir = test_dir.join("data");

        let tempo = Tempo::new(&data_dir).unwrap();

        tempo.scan_plugins().unwrap();

        Self {
            tempo,
            test_dir,
            data_dir,
        }
    }

    /// Creates a Tempo instance, performs initial plugin scan, creates and adds folder
    pub fn new_with_folder(prefix: &str, username: &str) -> (Self, std::path::PathBuf) {
        let s = Self::new(prefix);

        let folder = s.test_dir.join("folder");

        expect_create_dir_all(&folder);

        s.tempo
            .create_folder(&folder)
            .expect("failed to create test tempo folder");

        s.tempo.add_folder(&folder, username).unwrap_or_else(|_| {
            panic!(
                "failed to add test tempo folder {}",
                folder.to_string_lossy()
            )
        });

        (s, folder)
    }
}

#[test]
fn test_new_tempo() {
    use crate::Tempo;

    let dir = get_temp_dir("test_setup_loading");
    let data_dir = dir.join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    {
        Tempo::new(&data_dir).expect("failed to create new Tempo instance");
    }
    {
        Tempo::new(&data_dir).expect("failed to load tempo a second time");
    }
}

#[test]
fn test_folder() {
    use crate::Tempo;

    let dir = get_temp_dir("test_folder");
    let data_dir = dir.join("data");
    let folder = dir.join("folder");
    let folder2 = dir.join("folder2");

    for d in [&data_dir, &folder, &folder2] {
        std::fs::create_dir_all(d)
            .unwrap_or_else(|_| panic!("failed to create test directory {}", d.to_string_lossy()));
    }

    let t = Tempo::new(&data_dir).expect("failed to create tempo");

    // adding folder
    t.create_folder(&folder)
        .expect("failed to create a tempo folder");

    // this should fail since we haven't scanned plugins yet
    t.add_folder(&folder, "test")
        .expect_err("shouldn't be able to add a folder before scanning plugins");

    // cant add empty folder
    t.add_folder(&folder2, "test")
        .expect_err("shouldn't be able to add an empty folder");

    t.scan_plugins().expect("failed to scan plugins");

    t.add_folder(&folder, "test")
        .expect("failed to add a folder after scanning plugins");
}

#[test]
fn test_channel() {
    let (
        Testpo {
            tempo,
            test_dir: _,
            data_dir: _,
        },
        folder,
    ) = Testpo::new_with_folder("test_note", "test");

    let folder = tempo
        .folder(&folder)
        .unwrap_or_else(|_| panic!("failed to load test folder {}", folder.to_string_lossy()));

    folder
        .channel(None)
        .expect("failed to retrieve global channel");
    folder
        .channel(Some(&crate::misc::new_ulid()))
        .expect_err("shouldn't be able to retrieve nonexistent channel");
    let channel = folder
        .create_channel("test")
        .expect("failed to create channel");

    let ulid = channel
        .ulid()
        .expect("failed to get ulid of created channel");

    folder
        .channel(Some(&ulid))
        .expect("failed to retrieve created channel with ulid");
}

#[test]
fn test_note() {
    use crate::shared::NewNote;

    let (
        Testpo {
            tempo,
            test_dir: _,
            data_dir: _,
        },
        folder,
    ) = Testpo::new_with_folder("test_note", "test");

    let global = tempo
        .folder(&folder)
        .expect("failed to retrieve tempo folder")
        .channel(None)
        .expect("failed to retrieve global channel");

    let first_note = global
        .create_note(NewNote {
            body: "hi".into(),
            reply_ulid: None,
            attachment: None,
        })
        .expect("failed to create note in global with no reply, no attachment");

    global
        .create_note(NewNote {
            body: "hi there".into(),
            reply_ulid: Some(crate::misc::new_ulid()),
            attachment: None,
        })
        .expect_err("shouldn't be able to reply to nonexistent note in global");

    let _second_note = global
        .create_note(NewNote {
            body: "how are you".into(),
            reply_ulid: Some(first_note.ulid().to_string()),
            attachment: None,
        })
        .expect("failed to create reply note in global with no attachment");
}
