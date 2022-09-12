#[derive(Debug)]
pub enum PatchConfigError {
    SourceNotDirectory(String),
    TargetAlreadyExists(String),
    ReadSourceDirectoryFailed(String),
    ReadSourceDirectoryEntryFailed(String),
    SourceFileNameInvalid(String),
    CreateTargetDirectoryFailed(String),
    ReadSourceFileFailed(String),
    WriteTargetFileFailed(String),
    NoArchiveFile(String),
    OpenArchiveFailed(String),
    ReadArchiveFailed(String),
    MetadataFailed(String),
    MetadataDirectoryFailed(String),
    WriteMetadataFailed(String),
    ArchiveContainsDirectory(String),
}

impl ToString for PatchConfigError {
    fn to_string(&self) -> String {
        match &self {
            PatchConfigError::SourceNotDirectory(s) => s,
            PatchConfigError::TargetAlreadyExists(s) => s,
            PatchConfigError::ReadSourceDirectoryFailed(s) => s,
            PatchConfigError::ReadSourceDirectoryEntryFailed(s) => s,
            PatchConfigError::SourceFileNameInvalid(s) => s,
            PatchConfigError::CreateTargetDirectoryFailed(s) => s,
            PatchConfigError::ReadSourceFileFailed(s) => s,
            PatchConfigError::WriteTargetFileFailed(s) => s,
            PatchConfigError::NoArchiveFile(s) => s,
            PatchConfigError::OpenArchiveFailed(s) => s,
            PatchConfigError::ReadArchiveFailed(s) => s,
            PatchConfigError::MetadataFailed(s) => s,
            PatchConfigError::MetadataDirectoryFailed(s) => s,
            PatchConfigError::WriteMetadataFailed(s) => s,
            PatchConfigError::ArchiveContainsDirectory(s) => s,
        }
        .clone()
    }
}
