use crate::{constants::*, error::PatchConfigError, fsobject::*, process_archive::*};

use std::path::{Path, PathBuf};

pub fn process_new_file<P>(
    source_file_path: P,
    target_file_path: P,
    object_name: &str,
) -> Result<Option<FSObject>, PatchConfigError>
where
    P: AsRef<Path>,
{
    // Handle ECO archives
    if let Some(extension) = source_file_path.as_ref().extension() {
        // Ignore DAT files, they will be processed when their HED is reached
        if extension == ARCHIVE_DATA_EXTENSION {
            return Ok(None);
        } else if extension == ARCHIVE_METADATA_EXTENSION {
            // Make a dir with the extension .archive instead
            if let (Some(target_parent), Some(target_stem)) = (
                target_file_path.as_ref().parent(),    /* Get containing path */
                target_file_path.as_ref().file_stem(), /* Get file name without extension */
            ) {
                // Make a target dir as <path>/<archive name>.<new archive extension>
                let mut target_dir = PathBuf::new();
                target_dir.push(target_parent);
                target_dir.push(target_stem);
                target_dir.set_extension(UNPACKED_ARCHIVE_EXTENSION);

                if let Some(object_name) = object_name.split('.').next() {
                    return process_new_archive(
                        source_file_path.as_ref(),
                        &target_dir,
                        object_name,
                    );
                }
            }
        }
    }

    // To avoid having to read the file twice, read it all into memory, hash
    // it, and then write the data to the target file.
    let data = std::fs::read(&source_file_path).map_err(|why| {
        PatchConfigError::ReadSourceFileFailed(format!(
            "Failed to read file {}: {}",
            source_file_path.as_ref().to_string_lossy(),
            why
        ))
    })?;

    let file_info = File::new(object_name, &data);

    std::fs::write(&target_file_path, &data).map_err(|why| {
        PatchConfigError::WriteTargetFileFailed(format!(
            "Failed to write file {}: {}",
            target_file_path.as_ref().to_string_lossy(),
            why
        ))
    })?;

    Ok(Some(FSObject::File(file_info)))
}
