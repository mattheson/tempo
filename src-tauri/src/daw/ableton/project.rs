use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use log::{error, info, warn};

use crate::{
    daw::{ableton::ProjectFileRefWriter, plugin::PluginType},
    db::{PluginNameVendor, SharedDb},
    file::{add_file_with_filename, add_referenced_file},
    misc::{
        extract_file_extension, get_filename, hash_file, path_to_str, remove_file_extension,
        Result, TempoError,
    },
    shared::{FileErr, FileRef, MissingFileRef, PluginRef, PluginScan, ProjectFileRefScan},
    structure::{expect_valid_folder, get_file_path},
    types::{FileMeta, ProjectData},
};

use super::{als::AbletonFileRef, AbletonPluginRef, ProjectFileRefReader, ProjectPluginReader};

pub fn scan_filerefs(project: &Path) -> Result<ProjectFileRefScan> {
    if !project.exists() || !project.is_file() {
        return Err(TempoError::Ableton(format!(
            "Project {} does not exist or is not a file",
            project.to_string_lossy()
        )));
    }

    let project_dir = project.parent().ok_or(TempoError::Ableton(format!(
        "Could not find parent directory of project {}",
        project.to_string_lossy()
    )))?;

    // dont you just love 200 closures

    // expand a relative path into an absolute path
    let expand_rel = |rel: &Path| project_dir.join(PathBuf::from(rel));

    // turn relative into absolute
    let rel = |fr: &AbletonFileRef| expand_rel(&PathBuf::from(&fr.rel));

    // get absolute
    let abs = |fr: &AbletonFileRef| PathBuf::from(&fr.abs);

    // whether fileref exists
    let exists = |fr: &AbletonFileRef| -> std::result::Result<(), String> {
        if rel(fr).exists() || abs(fr).exists() {
            Ok(())
        } else {
            Err("File does not exist".into())
        }
    };

    // whether we can read fileref
    let readable = |fr: &AbletonFileRef| -> std::result::Result<PathBuf, String> {
        let paths = (rel(fr), abs(fr));
        let res = (fs::File::open(&paths.0), fs::File::open(&paths.1));
        match res {
            (Ok(_), _) => Ok(paths.0),
            (_, Ok(_)) => Ok(paths.1),
            _ => Err("Could not read file, check file permissions".into()),
        }
    };

    // returns readable path for file, otherwise error
    let check_fr = |fr: &AbletonFileRef| -> std::result::Result<PathBuf, String> {
        exists(fr)?;
        readable(fr)
    };

    let mut found: HashSet<AbletonFileRef> = HashSet::new();

    let mut scan = ProjectFileRefScan {
        ok: HashSet::new(),
        missing: HashSet::new(),
    };

    for fr in ProjectFileRefReader::new(project)?.get_unique()? {
        if found.contains(&fr) {
            continue;
        }

        found.insert(fr.clone());

        match check_fr(&fr) {
            Ok(_) => scan.ok.insert(FileRef {
                rel: fr.rel,
                abs: fr.abs,
            }),
            Err(e) => scan.missing.insert(MissingFileRef {
                file: FileRef {
                    rel: fr.rel,
                    abs: fr.abs,
                },
                err: e,
            }),
        };
    }

    Ok(scan)
}

pub struct AbletonProjectPluginScan {
    refs: Vec<(AbletonPluginRef, Option<PluginNameVendor>)>,
    // { username : idx of missing }
    missing: HashMap<String, Vec<usize>>,
}

impl AbletonProjectPluginScan {
    pub fn new(path: &Path) -> Result<Self> {
        let mut found_plugs = HashSet::new();

        for plug in ProjectPluginReader::new(path)? {
            match plug {
                Ok(p) => {
                    found_plugs.insert(p);
                }
                Err(e) => error!("AbletonProjectScan::new(): error while scanning plugins: {e}"),
            }
        }

        Ok(Self {
            refs: found_plugs.into_iter().map(|p| (p, None)).collect(),
            missing: HashMap::new(),
        })
    }

    pub fn scan_db(&mut self, db: &SharedDb, username: &str) -> Result<()> {
        let mut missing: Vec<usize> = vec![];

        for (idx, (p, nv)) in &mut self.refs.iter_mut().enumerate() {
            let res = match db.get_ableton_plugin(p) {
                Ok(o) => o,
                Err(e) => {
                    error!("AbletonProjectScan::compare_db(): error while scanning plugin {:#?} for user {username}: {e}", p);
                    continue;
                }
            };
            match res {
                None => missing.push(idx),
                Some(db_nv) => {
                    if nv.is_none() {
                        *nv = Some(db_nv)
                    }
                }
            }
        }

        self.missing.insert(username.to_owned(), missing);

        Ok(())
    }

    pub fn done(self) -> PluginScan {
        let Self { refs, missing } = self;

        let plugins: Vec<PluginRef> = refs
            .into_iter()
            .map(|(plug, nv)| {
                let (plugin_type, mut name, mut vendor) = match plug {
                    AbletonPluginRef::Vst { id: _, name } => (PluginType::Vst, name, None),
                    AbletonPluginRef::Vst3 { fields: _, name } => (PluginType::Vst3, name, None),
                    AbletonPluginRef::Au {
                        id: _,
                        name,
                        manufacturer,
                    } => (PluginType::Au, name, manufacturer),
                };

                if let Some(PluginNameVendor { name: n, vendor: v }) = nv {
                    name = Some(n);
                    vendor = Some(v);
                }

                PluginRef {
                    plugin_type,
                    name: name.unwrap_or("Unknown".into()),
                    vendor: vendor.unwrap_or("Unknown".into()),
                }
            })
            .collect();

        PluginScan { plugins, missing }
    }

    pub fn done_ableton(self) -> Vec<AbletonPluginRef> {
        self.refs.into_iter().map(|(p, _)| p).collect()
    }
}

/// Copies an Ableton project **from a Tempo folder** into the given directory.
/// Creates the Files directory and copies all files referenced in the project.
/// The project will be copied into `dest`.
/// The Files directory will be created inside of `dest`.
pub fn copy_ableton_project(
    folder: &Path,
    project_sha256: &str,
    project_filename: &str,
    refs: &HashMap<String, String>,
    live_project: &Path,
) -> Result<Vec<FileErr>> {
    expect_valid_folder(folder)?;

    fs::copy(
        get_file_path(folder, project_sha256),
        live_project.join(project_filename),
    )?;

    // if this directory does not exist Ableton gets angry
    fs::create_dir_all(live_project.join("Ableton Project Info"))?;

    let files_dir = live_project.join("Files");

    let mut failures: Vec<FileErr> = vec![];

    if !refs.is_empty() {
        fs::create_dir_all(&files_dir)?;
        for (h, filename) in refs.iter() {
            // println!("{} {}", h, filename);
            match fs::copy(get_file_path(folder, h), files_dir.join(filename)) {
                Ok(_) => (),
                Err(e) => {
                    failures.push(FileErr {
                        filename: filename.to_owned(),
                        err: e.to_string(),
                    });
                }
            }
        }
    }

    Ok(failures)
}

/// Adds an Ableton project into a Tempo folder.
pub fn add_ableton_project(folder: &Path, username: &str, project: &Path) -> Result<String> {
    // something feels odd in this but it seems to work

    expect_valid_folder(folder)?;

    // make temporary copy of project and get output project we copy to
    let (copy, out) = prepare_ableton_project(project)?;

    info!(
        "created copy of project: {}, out: {}",
        copy.to_string_lossy(),
        out.to_string_lossy()
    );


    let writer = ProjectFileRefWriter::new(&copy, &out)?;

    // { hash : (filename to use in Files dir, original path) }
    let mut known: HashMap<String, (String, PathBuf)> = HashMap::new();

    // used filenames
    let mut used: HashSet<String> = HashSet::new();

    // previously handled file refs
    let mut found: HashMap<PathBuf, String> = HashMap::new();

    let create_rel_path = |filename: &str| format!("Files/{filename}");

    writer.edit_relative_paths(|fr| {
        let rel = project
            .parent()
            .ok_or(TempoError::Project(format!(
                "Error: Project path has no parent. This shouldn't happen. Project: {}",
                path_to_str(project)
            )))?
            .join(&fr.rel);
        let abs = PathBuf::from(&fr.abs);

        info!("resolving reference {:#?}", fr);
        info!("rel: {}, abs: {}", path_to_str(&rel), path_to_str(&abs));

        // try to find location of file
        let file = {
            if rel.exists() {
                if !rel.is_dir() {
                    Some(rel)
                } else {
                    warn!(
                        "relative path was directory in add_ableton_project(): {}",
                        path_to_str(&rel)
                    );
                    None
                }
            } else if abs.exists() {
                if !abs.is_dir() {
                    Some(abs)
                } else {
                    warn!(
                        "absolute path was directory in add_ableton_project(): {}",
                        path_to_str(&abs)
                    );
                    None
                }
            } else {
                None
            }
        };

        // just check for duplicate refs here
        if let Some(file) = file {
            // check if we've copied this file already
            if let Some(f) = found.get(&file) {
                Ok(Some(f.clone()))
            } else {
                let hash = hash_file(&file)?;
                info!("hash: {hash}");
                if let Some((f, _)) = known.get(&hash) {
                    Ok(Some(create_rel_path(f)))
                } else {
                    // otherwise get a unique filename for this file
                    let mut filename = get_filename(&file)?;
                    info!("filename: {filename}");
                    if used.contains(&filename) {
                        let (no_ext, ext) = extract_file_extension(&filename);
                        let mut dup: usize = 1;
                        while used.contains(&filename) {
                            filename = format!("{no_ext}-{dup}{}", {
                                if let Some(e) = ext.as_ref() {
                                    format!(".{e}")
                                } else {
                                    "".into()
                                }
                            });
                            dup += 1;
                        }
                    }
                    let files_path = create_rel_path(&filename);
                    known.insert(hash, (filename.clone(), file.clone()));
                    used.insert(filename.clone());
                    found.insert(file, files_path.clone());

                    Ok(Some(files_path))
                }
            }
        } else {
            warn!(
                "failed to find FileRef while copying project, skipping: {:#?}",
                fr
            );
            Ok(None)
        }
    })?;

    drop(used);
    drop(found);

    // now we've adjusted relative filerefs to point into Files dir
    // we need to go through and copy all files into shared folder

    let mut file_info_refs: HashMap<String, String> = HashMap::new();

    for (orig_hash, (filename_to_use, file_path)) in known.into_iter() {
        let added_hash = add_referenced_file(folder, username, &file_path)?;

        if added_hash != orig_hash {
            return Err(TempoError::Folder(format!(
                "A file referenced in the project file {} has changed while Tempo was copying it. Please save the project and try sending it again. File that changed: {}",
                project.to_string_lossy(),
                file_path.to_string_lossy()
            )));
        }

        file_info_refs.insert(added_hash, filename_to_use);
    }

    // scan plugins
    let plugins = AbletonProjectPluginScan::new(&copy)?.done_ableton();

    add_file_with_filename(
        folder,
        username,
        &out,
        &get_filename(project)?,
        FileMeta::Project(ProjectData::Ableton {
            refs: file_info_refs,
            plugins,
        }),
    )
}

/// Prepares to add an Ableton project to a Tempo folder.
/// We create a copy of the project, and create a destination file for our modified version of the project file.
/// Returns `(path to copy of project, output project path)`
fn prepare_ableton_project(project: &Path) -> Result<(PathBuf, PathBuf)> {
    let filename = remove_file_extension(&get_filename(project)?);

    let mut copy_filename = format!("[tempo copy] {filename}.als");
    let mut out_filename = format!("[tempo output] {filename}.als");

    let mut dup: usize = 1;

    let (src, out) = loop {
        let src = std::env::temp_dir().join(&copy_filename);
        let out = std::env::temp_dir().join(&out_filename);

        if src.exists() || out.exists() {
            copy_filename = format!("[tempo copy] {filename}-{dup}.als");
            out_filename = format!("[tempo output] {filename}-{dup}.als");
            dup += 1;
        } else {
            break (src, out);
        }
    };

    fs::copy(project, &src)?;

    if out.exists() {
        return Err(TempoError::Folder(format!(
            "Output project file {out_filename} already exists"
        )));
    }

    fs::File::create(&out)?;

    Ok((src, out))
}

// TODO add tests here again