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

// parsing and modification of ableton project files

use flate2::{bufread::GzDecoder, write::GzEncoder, Compression};
use log::warn;
use quick_xml::{
    escape::unescape,
    events::{
        attributes::{AttrError, Attribute, Attributes},
        BytesStart, Event,
    },
    name::QName,
    Writer,
};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::{
    daw::{ableton::AbletonPluginRef, macos::AudioUnitId},
    misc::{Result, TempoError},
};

// TODO create some generalized interface for reading project file info to add other daws later on
// would be nice as a standalone separate library

// TODO maybe serde would be nice for matching filerefs?
// might avoid the weirdness with match_event_in_fileref

/*
overview:

Tempo has two big features which rely on parsing/modifying Ableton project files:

1. automatic collect all and save
  - requires us to parse file references
  - we modify relative paths to point to Tempo's Files directory
  - we automatically copy referenced files into the shared folder
2. plugin compatibility checks
  - just requires reading of plugin ids, we compare these against users' Ableton plugin dbs

when a user adds a project in a shared folder, the following happens:
- plugin compatibility scanning
- scanning of file references
  - referenced files are copied into the folder
  - relative paths of FileRefs are adjusted to point into Files directory

when a user makes a local copy of a project from a shared folder, the following happens:
- a new directory is created, we also create a Files directory inside of the directory
  - in Ableton terminology, this directory we create is a Project
- the project file itself is copied into the directory
  - in Ableton terminology, we copy the Set into the Project
- we copy all of the referenced files as provided in the FileInfo into the Files directory
  - the Files directory kind of acts like the Samples directory you get when you collect all and save

-----------------------------------------------------------------------------------------------------
notes:

as of now there are three actions we perform with Ableton project files:

1. reading FileRefs
   - to verify that all file references exist
2. reading plugins
   - for plugin compatibility checking features
3. editing FileRefs
   - before any project is copied into a folder, Tempo updates all relevant FileRefs to point into a `./Files` directory
   - when the project is copied out all the referenced files are copied into the Files folder with appropriate filenames as specified by the project's FileInfo

currently three separate types have been created for these three use cases, namely:
1. ProjectFileRefReader
   - Iterator impl
2. ProjectPluginReader
   - Iterator impl
3. ProjectFileRefWriter
   - no Iterator impl
   - user calls edit_relative() with a FnMut to edit particular FileRefs

-----------------------------------------------------------------------------------------------------
Ableton project file schema notes:

as of now we are interested in reading two pieces of information:
1. FileRefs
2. plugin ids

and we are only interested in editing FileRefs

right now Tempo supports parsing semi-recent Ableton project files. it seems like FileRefs and PluginInfos have had relatively stable schemas for the last few years.
i think Tempo probably supports project files created by late versions of Live 10? older versions of project files use an alternative FileRef schema.
i'm not sure what PluginInfos look like on older schemas, but i don't see how they could be too different.

Tempo is not too focused on supporting really old project file schemas, it will always target recent versions of Ableton.

-----------------------------------------------------------------------------------------------------
FileRefs:

we are interested in reading/editing FileRefs for audio samples and m4l plugins.
these are where broken references most often happen.

references to samples are within SampleRef tags, and m4l plugins are within MxPatchRef tags.
importantly, it seems that only the FileRef that is a DIRECT child of the SampleRef/MxPatchRef actually matters.
sometimes Ableton likes to put SourceContext or OriginalFileRef tags within SampleRefs/MxPatchRefs, these appear to be optional and we want to ignore them.

there seem to be FileRefs within FileRefs in some cases, such as Ableton's convolution reverb.
it doesnt seem like these crop up too often and aren't supported right now i think. it should not be too tricky to add these if needed.

-----------------------------------------------------------------------------------------------------
DRMed .aif files:

Ableton uses some sort of encryption/DRM on .aif files contained in Live packs.
it is possible to import these files using collect all and save into Ableton, so there is no problem with copying them around.
it probably breaks the EULA to try to decrypt these files, so Tempo will just copy them around like how Ableton does with collect all and save.

-----------------------------------------------------------------------------------------------------
Ableton Project Info:

when a directory named "Ableton Project Info" exists in a directory containing Ableton project files, this seems to turn the folder into a Live Project.
without creating this directory, Ableton does not like to load DRMed samples (from previous section)

-----------------------------------------------------------------------------------------------------
RelativePathType:

RelativePathType (i assume) determines what a RelativePath in a FileRef is relative to.
it seems like Value="3" means it's relative to the project file itself.

it looks like Value="5" means the path is relative to the root of a Live pack?
and Value="7" seems to mean it's relative to some other core directory of Ableton I think.

we only care about setting RelativePathType to the value which will make the relative path relative to the project file. again, i think this is Value="3"

these values probably differ with different schema versions

-----------------------------------------------------------------------------------------------------
PluginInfo:

Ableton supports three types of plugins: vst, vst3 and au.
Tempo does not modify plugin info in projects

each of these plugin types has their own unique identifiers:

vst:
- u32 identifier
- ableton likes to also keep track of the plugin name (probably since it's probable that two plugins might use the same id)

vst3:
- uses guids (16 byte id)

au:
- 3 u32s, plugin type/subtype/manufacturer

plugin device ids as found in Ableton's plugin db (plugins table) can be used to construct these format-specific ids

 */

type GzXmlReader = quick_xml::reader::Reader<BufReader<GzDecoder<BufReader<File>>>>;
type GzXmlWriter = quick_xml::Writer<GzEncoder<BufWriter<File>>>;

const VST_PLUGIN_INFO: &[u8] = b"VstPluginInfo";
const VST3_PLUGIN_INFO: &[u8] = b"Vst3PluginInfo";
const AU_PLUGIN_INFO: &[u8] = b"AuPluginInfo";

const UNIQUE_ID: &[u8] = b"UniqueId";
const PLUG_NAME: &[u8] = b"PlugName";
const UID: &[u8] = b"Uid";
const NAME: &[u8] = b"Name";
const MANUFACTURER: &[u8] = b"Manufacturer";

const COMPONENT_TYPE: &[u8] = b"ComponentType";
const COMPONENT_SUBTYPE: &[u8] = b"ComponentSubType";
const COMPONENT_MANUFACTURER: &[u8] = b"ComponentManufacturer";

/// FileRef extracted from an Ableton project file.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct AbletonFileRef {
    pub rel: String,
    pub abs: String,
    pub rel_type: String,
}

/// Creates a new `GzXMLReader`, used for reading Ableton project files.
fn new_gzxml_reader(project: &Path) -> Result<GzXmlReader> {
    if !project.exists() {
        return Err(TempoError::Ableton(format!(
            "Project file does not exist: {}",
            project.to_string_lossy()
        )));
    }

    let file = File::open(project)?;

    let buf_reader = BufReader::new(file);
    let gz_decoder = GzDecoder::new(buf_reader);
    let decompressed_reader = BufReader::new(gz_decoder);
    let mut xml_reader = quick_xml::reader::Reader::from_reader(decompressed_reader);

    xml_reader.config_mut().trim_text(true);
    Ok(xml_reader)
}

/// Creates a new `GzXMLWriter` which can be used to write a new Ableton project file.
/// This can be used with a companion `GzXMLReader` which reads from an existing project.
///
/// Desired changes can be made to the project file, and can be passed to the `GzXMLWriter`.
/// If you want to leave an event unmodified, you can just pass it to the `GzXMLWriter` untouched.
fn new_gzxml_writer(project: &Path) -> Result<GzXmlWriter> {
    let file = File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(project)?;
    let buf_writer = BufWriter::new(file);
    let gz_encoder = GzEncoder::new(buf_writer, Compression::default());
    let xml_writer = Writer::new(gz_encoder);

    Ok(xml_writer)
}

/// States that our project readers/writers can be in.
enum ProjectFileRefIterState {
    // we are inside a SampleRef (or MxPatchRef!!!) and are looking for FileRef
    InsideSampleRef,
    None,
}

/// Type which iterates over FileRefs in an Ableton project.
///
/// This only iterates over FileRefs that are immediate children of SampleRef or MxPatchRef.
pub struct ProjectFileRefReader {
    reader: GzXmlReader,
    buf: Vec<u8>,

    state: ProjectFileRefIterState,
    done: bool,
}

impl ProjectFileRefReader {
    /// Creates new `ProjectFileRefReader`, does a little validation to see that provided Ableton project is valid
    pub fn new(project: &Path) -> Result<Self> {
        // the reader will be placed right after the Ableton event afterwards

        let (reader, buf) = get_project_reader_and_validate(project)?;

        Ok(Self {
            reader,
            buf,
            done: false,
            state: ProjectFileRefIterState::None,
        })
    }

    fn handle_err(&mut self, e: quick_xml::Error) -> Option<Result<AbletonFileRef>> {
        self.done = true;
        Some(Err(TempoError::Ableton(format!(
            "XML error at byte {}: {e}",
            self.reader.buffer_position()
        ))))
    }

    /// Returns a set of unique FileRefs in this project.
    pub fn get_unique(self) -> Result<HashSet<AbletonFileRef>> {
        let mut s: HashSet<AbletonFileRef> = HashSet::new();

        for i in self.into_iter() {
            let i = i?;
            s.insert(i);
        }

        Ok(s)
    }
}

impl Iterator for ProjectFileRefReader {
    type Item = Result<AbletonFileRef>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        loop {
            self.buf.clear();

            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    match self.state {
                        ProjectFileRefIterState::None => {
                            if e.name().as_ref() == b"SampleRef"
                                || e.name().as_ref() == b"MxPatchRef"
                            {
                                self.state = ProjectFileRefIterState::InsideSampleRef;
                            }
                        }
                        ProjectFileRefIterState::InsideSampleRef => {
                            // return next fileref
                            if e.name().as_ref() == b"FileRef" {
                                // self.buf.clear();
                                self.state = ProjectFileRefIterState::None;
                                return Some(
                                    match match_fileref_read(&mut self.reader, &mut self.buf) {
                                        Ok(f) => Ok(f),
                                        Err(e) => {
                                            self.done = true;
                                            Err(e)
                                        }
                                    },
                                );
                            } else {
                                // we've found an opening tag for something that isn't a FileRef, we need to skip past it
                                // we only want to look for the FileRef that's an immediate child of the SampleRef
                                let e = e.to_owned();
                                let end = e.to_end();
                                // self.buf.clear();
                                match self.reader.read_to_end_into(end.name(), &mut self.buf) {
                                    Ok(_) => continue,
                                    Err(e) => return self.handle_err(e),
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => {
                    self.done = true;
                    return None;
                }
                Ok(_) => (),
                Err(e) => return self.handle_err(e),
            }
        }
    }
}

pub struct ProjectPluginReader {
    reader: GzXmlReader,
    buf: Vec<u8>,
    done: bool,
}

impl ProjectPluginReader {
    pub fn new(project: &Path) -> Result<Self> {
        let (reader, buf) = get_project_reader_and_validate(project)?;

        Ok(Self {
            reader,
            buf,
            done: false,
        })
    }

    fn handle_err(&mut self, e: quick_xml::Error) -> Option<Result<AbletonPluginRef>> {
        self.done = true;
        Some(Err(TempoError::Ableton(format!(
            "XML error at byte {}: {e}",
            self.reader.buffer_position()
        ))))
    }
}

impl Iterator for ProjectPluginReader {
    type Item = Result<AbletonPluginRef>;

    fn next(&mut self) -> Option<Self::Item> {
        // look for opening PluginInfo tags and call appropriate functions to match plugins

        if self.done {
            return None;
        }

        loop {
            self.buf.clear();

            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    VST_PLUGIN_INFO => {
                        break Some(match_vst_plugin_info(&mut self.reader, &mut self.buf))
                    }
                    VST3_PLUGIN_INFO => {
                        break Some(match_vst3_plugin_info(&mut self.reader, &mut self.buf))
                    }
                    AU_PLUGIN_INFO => {
                        break Some(match_au_plugin_info(&mut self.reader, &mut self.buf))
                    }
                    _ => (),
                },
                Ok(Event::Eof) => {
                    self.done = true;
                    break None;
                }
                Ok(_) => (),
                Err(e) => break self.handle_err(e),
            }
        }
    }
}

/// Type which iterates over FileRefs in an Ableton project.
/// Allows editing of relative paths in FileRefs.
///
/// This only iterates over FileRefs that are immediate children of SampleRef or MxPatchRef.
pub struct ProjectFileRefWriter {
    reader: GzXmlReader,
    writer: GzXmlWriter,
    buf: Vec<u8>,

    state: ProjectFileRefIterState,
}

impl ProjectFileRefWriter {
    /// Sets up a new ProjectFileRefWriter.
    /// Important note: you need to make sure `output` does not point to any sort of existing Ableton project from the user. Always create a copy.
    pub fn new(input: &Path, output: &Path) -> Result<Self> {
        let (reader, writer, buf) = get_project_reader_and_writer_and_validate(input, output)?;
        Ok(Self {
            reader,
            writer,
            buf,
            state: ProjectFileRefIterState::None,
        })
    }

    /// Takes a closure, iterates over FileRefs and calls the provided closure with the current FileRef.
    /// The closure should return a new relative path for the FileRef.
    ///
    /// If an `Err` is returned, this function will return the `Err` and immediately stop iterating over the FileRefs.
    /// If `None` is returned, the relative path will not be modified.
    pub fn edit_relative_paths<F>(self, mut f: F) -> Result<()>
    where
        F: FnMut(&AbletonFileRef) -> Result<Option<String>>,
    {
        let ProjectFileRefWriter {
            mut reader,
            mut writer,
            mut buf,
            mut state,
        } = self;

        loop {
            buf.clear();
            match reader.read_event_into(&mut buf) {
                Ok(ref e) => {
                    if let Event::Start(e) = e {
                        match state {
                            ProjectFileRefIterState::None => {
                                writer.write_event(Event::Start(e.clone()))?;
                                if e.name().as_ref() == b"SampleRef"
                                    || e.name().as_ref() == b"MxPatchRef"
                                {
                                    state = ProjectFileRefIterState::InsideSampleRef;
                                }
                            }
                            ProjectFileRefIterState::InsideSampleRef => {
                                if e.name().as_ref() == b"FileRef" {
                                    let e = e.to_owned();
                                    writer.write_event(Event::Start(e.clone()))?;
                                    // buf.clear();
                                    state = ProjectFileRefIterState::None;
                                    let mut fileref =
                                        match_fileref_write(&mut reader, &mut writer, &mut buf)?;
                                    let mut mutated = false;
                                    if let Some(new_rel) = f(&fileref)? {
                                        fileref.rel = new_rel;
                                        mutated = true;
                                    };
                                    writer
                                        .create_element("RelativePath")
                                        .with_attribute(("Value", fileref.rel.as_ref()))
                                        .write_empty()?
                                        .create_element("Path")
                                        .with_attribute(("Value", fileref.abs.as_ref()))
                                        .write_empty()?;
                                    writer
                                        .create_element("RelativePathType")
                                        .with_attribute((
                                            "Value", // see notes about RelativePathType, i think we always want a type of "3"
                                            if mutated { "3" } else { &fileref.rel_type },
                                        ))
                                        .write_empty()?;
                                    writer.write_event(Event::End(e.to_end()))?;
                                    // write Path, AbsolutePath and closing FileRef tags
                                } else {
                                    // found some kind of other opening tag, need to skip past it
                                    let e = e.to_owned();
                                    writer.write_event(Event::Start(e.clone()))?;
                                    let end = e.to_end();
                                    read_to_end_into_writer(
                                        &mut reader,
                                        &mut writer,
                                        end.name(),
                                        &mut buf,
                                    )?;
                                }
                            }
                        }
                    } else {
                        writer.write_event(e.clone())?;
                        if e == &Event::Eof {
                            return Ok(());
                        }
                    }
                }
                Err(e) => {
                    return Err(TempoError::Ableton(format!(
                        "XML error at byte {}: {e}",
                        reader.buffer_position()
                    )));
                }
            }
        }
    }
}

/// Essentially `quick_xml::reader::Reader::read_to_end_into()` but takes a writer to write all events into.
/// Caller needs to read opening tag and pass corresponding end tag.
/// Does not write opening tag. Writes all tags including closing tag.
fn read_to_end_into_writer(
    reader: &mut GzXmlReader,
    writer: &mut GzXmlWriter,
    end: QName,
    buf: &mut Vec<u8>,
) -> Result<()> {
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Eof) => {
                return Err(TempoError::Ableton(
                    "Unexpected EOF, possibly corrupt project".into(),
                ));
            }
            Ok(e) => {
                writer.write_event(e.clone())?;
                if let Event::End(e) = e {
                    if e.name() == end {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                return Err(TempoError::Ableton(format!(
                    "XML error at byte {}: {e}",
                    reader.buffer_position()
                )))
            }
        }
    }
}

/// Reads all of the **direct** empty children tags of the provided parent tag.
/// Calls the provided closure with the next empty child of the parent.
/// Places reader after closing parent tag.
///
/// This function should be called right after the opening parent tag is matched.
fn match_empty_children_of<F>(
    reader: &mut GzXmlReader,
    buf: &mut Vec<u8>,
    tag: &[u8],
    mut f: F,
) -> Result<()>
where
    F: FnMut(&BytesStart, &GzXmlReader) -> Result<()>,
{
    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::End(e)) if e.name().as_ref() == tag => break Ok(()),
            Ok(Event::End(e)) => warn!("read_next_empty_child_of(): encountered an Event::End which should not happen. found {} at byte {}", String::from_utf8_lossy(e.to_owned().name().as_ref()), reader.buffer_position()),
            Ok(Event::Start(e)) => {
                let e = e.to_owned();
                buf.clear();
                reader.read_to_end_into(e.to_end().name(), buf)?;
            }
            Ok(Event::Empty(e)) => f(&e, reader)?,
            Ok(Event::Eof) => return Err(TempoError::Ableton(format!("XML error: unexpected EOF found within {}", String::from_utf8_lossy(tag)))),
            Ok(_) => (),
            Err(e) => return Err(TempoError::from(e)),
        }
    }
}

/// Sets up `GzXMLReader` for reading an Ableton project.
/// Also sets up a buffer to read events into.
///
/// Does a little verification of the project beforehand, the reader will be placed immediately after the opening Ableton tag.
fn get_project_reader_and_validate(project: &Path) -> Result<(GzXmlReader, Vec<u8>)> {
    let mut r = new_gzxml_reader(project)?;
    r.config_mut().trim_text(true);

    let mut buf = Vec::new();

    match_decl(&mut r, &mut buf)?;
    match_ableton(&mut r, &mut buf)?;

    buf.clear();

    Ok((r, buf))
}

/// Returns whether given file is an Ableton project.
pub fn verify_project(project: &Path) -> Result<()> {
    match get_project_reader_and_validate(project) {
        Err(e) => Err(e),
        Ok(_) => Ok(()),
    }
}

fn get_project_reader_and_writer_and_validate(
    input: &Path,
    output: &Path,
) -> Result<(GzXmlReader, GzXmlWriter, Vec<u8>)> {
    let mut r = new_gzxml_reader(input)?;
    let mut w = new_gzxml_writer(output)?;

    let mut buf = Vec::new();

    w.write_event(match_decl(&mut r, &mut buf)?)?;
    w.write_event(match_ableton(&mut r, &mut buf)?)?;

    buf.clear();

    Ok((r, w, buf))
}

/// Matches the first Decl event.
fn match_decl<'a>(r: &mut GzXmlReader, buf: &'a mut Vec<u8>) -> Result<Event<'a>> {
    match r.read_event_into(buf) {
        Ok(Event::Decl(e)) => Ok(Event::Decl(e)),
        Ok(_) => Err(TempoError::Ableton(
            "Expected Decl event, found other event".into(),
        )),
        Err(e) => Err(TempoError::Ableton(format!(
            "Failed to read Decl event: {}",
            e
        ))),
    }
}

/// Returns whether the opening Ableton event appears to be valid.
fn match_ableton<'a>(r: &mut GzXmlReader, buf: &'a mut Vec<u8>) -> Result<Event<'a>> {
    // sort of TODO
    // we just check whether MajorVersion = 5 right now
    // don't know how reliable this is

    match r.read_event_into(buf) {
        Ok(Event::Start(e)) => {
            if e.name().as_ref() != b"Ableton" {
                return Err(TempoError::Ableton(format!(
                    "Failed to match Ableton event, found {}",
                    r.decoder().decode(e.name().as_ref())?
                )));
            }
            let mut found_schemachangecount = false;
            for a in e.attributes() {
                let a = map_attr_result(r, a)?;
                if a.key.as_ref() != b"MajorVersion" {
                    continue;
                }
                if a.value.as_ref() == b"5" {
                    found_schemachangecount = true;
                    break;
                } else {
                    return Err(TempoError::Ableton(format!(
                        "Ableton MajorVersion is not 5, found {} instead. \
                        This probably means you're using an unsupported newer or older version of Ableton. \
                        Sorry for the inconvenience.", r.decoder().decode(a.value.as_ref())?)));
                }
            }
            if !found_schemachangecount {
                Err(TempoError::Ableton(
                    "Could not find a MajorVersion in Ableton event".into(),
                ))
            } else {
                Ok(Event::Start(e))
            }
        }
        Ok(_) => Err(TempoError::Ableton(
            "Failed to read Ableton event, expected a Start event, found unexpected other event"
                .into(),
        )),
        Err(e) => Err(TempoError::Ableton(format!(
            "Failed to read Ableton event: {e}"
        ))),
    }
}

fn map_attr_result<'a>(
    r: &GzXmlReader,
    a: std::result::Result<Attribute<'a>, AttrError>,
) -> Result<Attribute<'a>> {
    a.map_err(|e| {
        TempoError::Ableton(format!(
            "XML attribute error at byte {}: {e}",
            r.buffer_position()
        ))
    })
}

/// Matches a single event found within a FileRef block and extracts relative and absolute path tags.
/// This does not handle matching the opening or closing FileRef tag, the caller must take care of this.
///
/// The RelativePathType event is also ignored.
///
/// Returns `Some` for all events which are not RelativePath or Path tags.
fn handle_event_in_fileref<'a>(
    reader: &GzXmlReader,
    event: quick_xml::Result<Event<'a>>,
    rel: &mut Option<String>,
    abs: &mut Option<String>,
    rel_type: &mut Option<String>,
) -> Result<Option<Event<'a>>> {
    let save_value = |tag: &[u8], val: String, opt: &mut Option<String>| {
        if let Some(val) = opt.as_ref() {
            Err(TempoError::Ableton(format!(
                "XML error: multiple {}s found in one FileRef at byte {}, last value: {val}",
                String::from_utf8_lossy(tag),
                reader.buffer_position()
            )))
        } else {
            *opt = Some(val);
            Ok(())
        }
    };

    const RELATIVEPATH: &[u8] = b"RelativePath";
    const PATH: &[u8] = b"Path";
    const RELATIVEPATHTYPE: &[u8] = b"RelativePathType";

    match event {
        Ok(Event::Empty(e)) => {
            if e.name().as_ref() == RELATIVEPATH {
                save_value(
                    RELATIVEPATH,
                    extract_value(reader, e.attributes(), RELATIVEPATH)?,
                    rel,
                )?;
                Ok(None)
            } else if e.name().as_ref() == PATH {
                save_value(PATH, extract_value(reader, e.attributes(), PATH)?, abs)?;
                Ok(None)
            } else if e.name().as_ref() == RELATIVEPATHTYPE {
                save_value(
                    RELATIVEPATHTYPE,
                    extract_value(reader, e.attributes(), RELATIVEPATHTYPE)?,
                    rel_type,
                )?;
                Ok(None)
            } else {
                Ok(Some(Event::Empty(e)))
            }
        }
        Ok(e) => Ok(Some(e)),
        Err(e) => Err(TempoError::from(e)),
    }
}

fn build_fileref(
    rel: Option<String>,
    abs: Option<String>,
    rel_type: Option<String>,
    reader: &GzXmlReader,
) -> Result<AbletonFileRef> {
    match (rel, abs, rel_type) {
        (Some(rel), Some(abs), Some(rel_type)) => Ok(AbletonFileRef { rel, abs, rel_type }),
        (a, b, c) => Err(TempoError::Ableton(format!(
            "Failed to build FileRef at byte {}, expected (rel, abs, rel_type), found ({:#?}, {:#?}, {:#?})",
            reader.buffer_position(), a, b, c
        ))),
    }
}

/// Matches in the relative and absolute path of a FileRef.
///
/// This should be called immediately after the opening FileRef tag is matched.
/// Will set the reader immediately after the closing FileRef tag.
fn match_fileref_read(reader: &mut GzXmlReader, buf: &mut Vec<u8>) -> Result<AbletonFileRef> {
    let mut rel: Option<String> = None;
    let mut abs: Option<String> = None;
    let mut rel_type: Option<String> = None;

    loop {
        buf.clear();
        let event = reader.read_event_into(buf);
        if let Some(Event::End(ref e)) =
            handle_event_in_fileref(reader, event, &mut rel, &mut abs, &mut rel_type)?
        {
            if e.name().as_ref() == b"FileRef" {
                break;
            }
        }
    }

    build_fileref(rel, abs, rel_type, reader)
}

/// Matches in the relative and absolute path of a FileRef.
///
/// This should be called immediately after the opening FileRef tag is matched.
/// Will set the reader immediately after the closing FileRef tag.
///
/// This will write all tags within the FileRef to the writer except for the:
/// - opening/closing FileRef tags
/// - RelativePath and Path tags
/// - the RelativePathType tag
fn match_fileref_write(
    reader: &mut GzXmlReader,
    writer: &mut GzXmlWriter,
    buf: &mut Vec<u8>,
) -> Result<AbletonFileRef> {
    let mut rel: Option<String> = None;
    let mut abs: Option<String> = None;
    let mut rel_type: Option<String> = None;

    loop {
        buf.clear();
        let event = reader.read_event_into(buf);
        match handle_event_in_fileref(reader, event, &mut rel, &mut abs, &mut rel_type) {
            Err(e) => return Err(e),
            Ok(Some(e)) => {
                if let Event::End(ref e) = e {
                    if e.name().as_ref() == b"FileRef" {
                        break;
                    }
                } else {
                    writer.write_event(e)?;
                }
            }
            Ok(None) => (),
        }
    }

    build_fileref(rel, abs, rel_type, reader)
}

/// Matches a VstPluginInfo.
///
/// This should be called immediately after the opening VstPluginInfo tag is matched.
/// Will set the reader immediately after the closing VstPluginInfo tag.
fn match_vst_plugin_info(reader: &mut GzXmlReader, buf: &mut Vec<u8>) -> Result<AbletonPluginRef> {
    // we want the PlugName and UniqueId. both are empty events which are nice
    let mut id: Option<u32> = None;
    let mut name: Option<String> = None;

    let mut set_id = |val: String, reader: &GzXmlReader| -> Result<()> {
        if let Some(id) = id.as_ref() {
            return Err(TempoError::Ableton(format!("XML error: found multiple UniqueIds in VstPluginInfo at byte {}, previous value: {id}", reader.buffer_position())));
        }
        id = Some(val.parse::<u32>()?);
        Ok(())
    };

    let mut set_name = |val: String, reader: &GzXmlReader| -> Result<()> {
        if let Some(name) = name.as_ref() {
            return Err(TempoError::Ableton(format!("XML error: found multiple PlugNames in VstPluginInfo at byte {}, previous value: {name}", reader.buffer_position())));
        }
        name = Some(val);
        Ok(())
    };

    buf.clear();

    match_empty_children_of(reader, buf, VST_PLUGIN_INFO, |e, reader| {
        match e.name().as_ref() {
            UNIQUE_ID => set_id(extract_value(reader, e.attributes(), UNIQUE_ID)?, reader)?,
            PLUG_NAME => set_name(extract_value(reader, e.attributes(), PLUG_NAME)?, reader)?,
            _ => (),
        }
        Ok(())
    })?;

    Ok(AbletonPluginRef::Vst {
        id: match id {
            None => {
                return Err(TempoError::Ableton(format!(
                    "Could not find UniqueId in VstPluginInfo at byte {}",
                    reader.buffer_position()
                )))
            }
            Some(id) => id,
        },
        name,
    })
}

fn match_vst3_plugin_info(reader: &mut GzXmlReader, buf: &mut Vec<u8>) -> Result<AbletonPluginRef> {
    // we look for the Uid block and the Name tag. we want the ones which are direct children of the Vst3PluginInfo

    let mut id: Option<[i32; 4]> = None;
    let mut name: Option<String> = None;

    let mut set_id = |val: [i32; 4], reader: &GzXmlReader| -> Result<()> {
        if let Some(id) = id.as_ref() {
            return Err(TempoError::Ableton(format!(
                "XML error: found multiple Uids in VstPluginInfo at byte {}, previous value: {:#?}",
                reader.buffer_position(),
                id
            )));
        }
        id = Some(val);
        Ok(())
    };

    let mut set_name = |val: String, reader: &GzXmlReader| -> Result<()> {
        if let Some(name) = name.as_ref() {
            return Err(TempoError::Ableton(format!("XML error: found multiple Names in VstPluginInfo at byte {}, previous value: {name}", reader.buffer_position())));
        }
        name = Some(val);
        Ok(())
    };

    loop {
        buf.clear();
        match reader.read_event_into(buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == NAME => {
                set_name(extract_value(reader, e.attributes(), NAME)?, reader)?
            }
            Ok(Event::Start(e)) => {
                if e.name().as_ref() == UID {
                    set_id(match_vst3_uid(reader, buf)?, reader)?;
                } else {
                    let e = e.to_owned();
                    reader.read_to_end_into(e.to_end().name(), buf)?;
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == VST3_PLUGIN_INFO {
                    break;
                } else {
                    warn!("match_vst3_plugin_info(): found Event::End which should not happen, found {} at byte {}", String::from_utf8_lossy(e.name().as_ref()), reader.buffer_position());
                }
            }
            Ok(_) => (),
            Err(e) => return Err(TempoError::from(e)),
        }
    }

    Ok(AbletonPluginRef::Vst3 {
        fields: match id {
            None => {
                return Err(TempoError::Ableton(format!(
                    "Could not find UniqueId in VstPluginInfo at byte {}",
                    reader.buffer_position()
                )))
            }
            Some(id) => id,
        },
        name,
    })
}

/// Matches a Uid inside of a Vst3PluginInfo.
/// Should be called after the opening Uid tag is matched.
fn match_vst3_uid(reader: &mut GzXmlReader, buf: &mut Vec<u8>) -> Result<[i32; 4]> {
    let mut data: [i32; 4] = [0, 0, 0, 0];

    // match Fields tags
    for (i, v) in data.iter_mut().enumerate() {
        buf.clear();
        let curr_tag = format!("Fields.{i}");
        match reader.read_event_into(buf) {
            Ok(e) => match e {
                Event::Empty(e) if e.name().as_ref() == curr_tag.as_bytes() => {
                    *v = extract_value(reader, e.attributes(), curr_tag.as_bytes())?
                        .parse::<i32>()?;
                }
                e => {
                    return Err(TempoError::Ableton(format!(
                        "Expected Fields.[0..=3] inside of Uid, found {:#?} at byte {}",
                        e,
                        reader.buffer_position()
                    )))
                }
            },
            Err(e) => return Err(TempoError::from(e)),
        }
    }

    // closing tag
    match reader.read_event_into(buf) {
        Ok(e) => match e {
            Event::End(e) if e.name().as_ref() == UID => Ok(data),
            e => Err(TempoError::Ableton(format!(
                "Expected closing Uid tag, found {:#?} at byte {}",
                e,
                reader.buffer_position()
            ))),
        },
        Err(e) => Err(TempoError::from(e)),
    }
}

fn match_au_plugin_info(reader: &mut GzXmlReader, buf: &mut Vec<u8>) -> Result<AbletonPluginRef> {
    // we match the ComponentType, ComponentSubType, ComponentManufacturer, Name, and Manufacturer tags
    // again we look for the ones which are direct children of the AuPluginInfo
    // all are empty tags which are nice

    let mut au_type: Option<u32> = None;
    let mut au_subtype: Option<u32> = None;
    let mut au_manufacturer: Option<u32> = None;

    let mut name: Option<String> = None;
    let mut manufacturer: Option<String> = None;

    // TODO find a more succinct way of doing this without these closures

    let mut set_au_type = |val: u32, reader: &GzXmlReader| {
        if let Some(au_type) = au_type.as_ref() {
            return Err(TempoError::Ableton(format!(
                "Found multiple ComponentTypes in AuPluginInfo at byte {}, previous value: {au_type}, found value: {val}",
                reader.buffer_position()
            )));
        }
        au_type = Some(val);
        Ok(())
    };

    let mut set_au_subtype = |val: u32, reader: &GzXmlReader| {
        if let Some(au_subtype) = au_subtype.as_ref() {
            return Err(TempoError::Ableton(format!(
                "Found multiple ComponentSubTypes in AuPluginInfo at byte {}, previous value: {au_subtype}, found value: {val}",
                reader.buffer_position()
            )));
        }
        au_subtype = Some(val);
        Ok(())
    };

    let mut set_au_manufacturer = |val: u32, reader: &GzXmlReader| {
        if let Some(au_manufacturer) = au_manufacturer.as_ref() {
            return Err(TempoError::Ableton(format!(
                "Found multiple ComponentManufacturers in AuPluginInfo at byte {}, previous value: {au_manufacturer}, found value: {val}",
                reader.buffer_position()
            )));
        }
        au_manufacturer = Some(val);
        Ok(())
    };

    let mut set_name = |val: String, reader: &GzXmlReader| {
        if let Some(name) = name.as_ref() {
            return Err(TempoError::Ableton(format!(
                "Found multiple Names in AuPluginInfo at byte {}, previous value: {name}, found value: {val}",
                reader.buffer_position()
            )));
        }
        name = Some(val);
        Ok(())
    };

    let mut set_manufacturer = |val: String, reader: &GzXmlReader| {
        if let Some(manufacturer) = manufacturer.as_ref() {
            return Err(TempoError::Ableton(format!(
                "Found multiple Manufacturers in AuPluginInfo at byte {}, previous value: {manufacturer}, found value: {val}",
                reader.buffer_position()
            )));
        }
        manufacturer = Some(val);
        Ok(())
    };

    buf.clear();

    match_empty_children_of(reader, buf, AU_PLUGIN_INFO, |e, reader| {
        match e.name().as_ref() {
            COMPONENT_TYPE => set_au_type(
                extract_value(reader, e.attributes(), COMPONENT_TYPE)?.parse::<u32>()?,
                reader,
            ),
            COMPONENT_SUBTYPE => set_au_subtype(
                extract_value(reader, e.attributes(), COMPONENT_SUBTYPE)?.parse::<u32>()?,
                reader,
            ),
            COMPONENT_MANUFACTURER => set_au_manufacturer(
                extract_value(reader, e.attributes(), COMPONENT_MANUFACTURER)?.parse::<u32>()?,
                reader,
            ),
            NAME => set_name(extract_value(reader, e.attributes(), NAME)?, reader),
            MANUFACTURER => {
                set_manufacturer(extract_value(reader, e.attributes(), MANUFACTURER)?, reader)
            }
            _ => Ok(()),
        }
    })?;

    Ok(AbletonPluginRef::Au {
        id: AudioUnitId {
            au_type: match au_type {
                Some(v) => v,
                None => {
                    return Err(TempoError::Ableton(format!(
                        "Could not find ComponentType in AuPluginInfo at byte {}",
                        reader.buffer_position()
                    )))
                }
            },
            au_subtype: match au_subtype {
                Some(v) => v,
                None => {
                    return Err(TempoError::Ableton(format!(
                        "Could not find ComponentSubType in AuPluginInfo at byte {}",
                        reader.buffer_position()
                    )))
                }
            },
            manufacturer: match au_manufacturer {
                Some(v) => v,
                None => {
                    return Err(TempoError::Ableton(format!(
                        "Could not find ComponentManufacturer in AuPluginInfo at byte {}",
                        reader.buffer_position()
                    )))
                }
            },
        },
        name,
        manufacturer,
    })
}

/// Helper for extracting Value attribute from Empty events in Ableton projects.
fn extract_value(reader: &GzXmlReader, attrs: Attributes, name: &[u8]) -> Result<String> {
    let mut val: Option<String> = None;

    for a in attrs {
        let a = map_attr_result(reader, a)?;

        if a.key.as_ref() == b"Value" {
            val = Some(unescape(&reader.decoder().decode(&a.value)?)?.to_string());
            break;
        }
    }

    match val {
        None => Err(TempoError::Ableton(format!(
            "XML error: Failed to find Value attribute in {} tag at byte {}",
            String::from_utf8_lossy(name),
            reader.buffer_position()
        ))),
        Some(v) => Ok(v),
    }
}
