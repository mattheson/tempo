pub enum ObjectType {
    Text,
    AudioFile,
    AbletonProject,
    File
}

pub enum ObjectData {
    Text(TextObjectData),
    AudioFile(TextObjectData),
    AbletonProject(AbletonProjectObjectData),
    File(FileObjectData),
}

pub struct TextObjectData {

}

pub struct AudioFileObjectData {
    // Add fields as needed
}

pub struct AbletonProjectObjectData {
    // Add fields as needed
}

pub struct FileObjectData {
    // Add fields as needed
}

impl Object {

}
