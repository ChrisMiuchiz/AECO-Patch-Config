use std::path::{Path, PathBuf};

pub mod error;
use error::PatchConfigError;

pub mod fsobject;
pub mod status;

mod constants;
mod process_archive;
mod process_directory;
mod process_file;
use process_directory::process_dir;

const PATCH_DIR_NAME: &str = "patch";
const METADATA_DIR_NAME: &str = "meta";

pub fn generate_config<P>(
    source_dir: P,
    target_dir: P,
    maintenance: bool,
) -> Result<(), PatchConfigError>
where
    P: AsRef<Path>,
{
    // The source must already exist and must be a directory
    if !source_dir.as_ref().is_dir() {
        return Err(PatchConfigError::SourceNotDirectory(format!(
            "Source is not a directory: {}",
            source_dir.as_ref().to_string_lossy()
        )));
    }

    // The target directory shouldn't exist yet
    if target_dir.as_ref().exists() {
        return Err(PatchConfigError::TargetAlreadyExists(format!(
            "Target already exists: {}",
            target_dir.as_ref().to_string_lossy()
        )));
    }

    if let Err(why) = std::fs::create_dir(&target_dir) {
        return Err(PatchConfigError::CreateTargetDirectoryFailed(format!(
            "Unable to create target directory {}: {}",
            target_dir.as_ref().to_string_lossy(),
            why
        )));
    }

    let patch_dir_name = PATCH_DIR_NAME;
    let mut patch_dir = PathBuf::new();
    patch_dir.push(&target_dir);
    patch_dir.push(patch_dir_name);

    if let Err(why) = std::fs::create_dir(&patch_dir) {
        return Err(PatchConfigError::CreateTargetDirectoryFailed(format!(
            "Unable to create target directory {}: {}",
            patch_dir.to_string_lossy(),
            why
        )));
    }

    let dir_obj = process_dir(&source_dir.as_ref(), &patch_dir.as_ref(), patch_dir_name)?;

    let patch_json = match serde_json::to_string(&dir_obj) {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::MetadataFailed(format!(
                "Failed to serialize metadata: {}",
                why
            )));
        }
    };

    let metadata_dir_name = METADATA_DIR_NAME;
    let mut metadata_dir = PathBuf::new();
    metadata_dir.push(&target_dir);
    metadata_dir.push(metadata_dir_name);

    if let Err(why) = std::fs::create_dir(&metadata_dir) {
        return Err(PatchConfigError::MetadataDirectoryFailed(format!(
            "Unable to create metadata directory {}: {}",
            metadata_dir.to_string_lossy(),
            why
        )));
    }

    let mut patch_data_path = PathBuf::new();
    patch_data_path.push(&metadata_dir);
    patch_data_path.push("patchlist.json");

    if let Err(why) = std::fs::write(&patch_data_path, &patch_json) {
        return Err(PatchConfigError::WriteMetadataFailed(format!(
            "Unable to write metadata file {}: {}",
            patch_data_path.to_string_lossy(),
            why
        )));
    }

    let server_status = if maintenance {
        status::ServerStatus::Maintenance
    } else {
        status::ServerStatus::Online
    };

    let status_json = match serde_json::to_string(&server_status) {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::MetadataFailed(format!(
                "Failed to serialize metadata: {}",
                why
            )));
        }
    };

    let mut status_data_path = PathBuf::new();
    status_data_path.push(&metadata_dir);
    status_data_path.push("status.json");

    if let Err(why) = std::fs::write(&status_data_path, &status_json) {
        return Err(PatchConfigError::WriteMetadataFailed(format!(
            "Unable to write metadata file {}: {}",
            patch_data_path.to_string_lossy(),
            why
        )));
    }

    // println!("{obj:?}");

    Ok(())
}
