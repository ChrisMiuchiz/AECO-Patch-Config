use crate::{constants::*, error::PatchConfigError, fsobject::*, process_directory::*};
use rayon::prelude::*;
use std::path::Path;

pub fn process_new_archive<P>(
    source_hed_path: P,
    target_dir_path: P,
    object_name: &str,
) -> Result<Option<FSObject>, PatchConfigError>
where
    P: AsRef<Path>,
{
    // The HED is provided as an argument, so derive the path to its DAT
    let mut source_dat_path = source_hed_path.as_ref().to_path_buf();
    source_dat_path.set_extension(ARCHIVE_DATA_EXTENSION);

    let source_dat_path: &Path = source_dat_path.as_ref();
    let target_dir_path: &Path = target_dir_path.as_ref();
    let source_hed_path: &Path = source_hed_path.as_ref();

    if !source_hed_path.exists() {
        return Err(PatchConfigError::NoArchiveFile(format!(
            "Missing archive component {}",
            source_hed_path.to_string_lossy()
        )));
    }

    if !source_dat_path.exists() {
        return Err(PatchConfigError::NoArchiveFile(format!(
            "Missing archive component {}",
            source_dat_path.to_string_lossy()
        )));
    }

    // Now that we have the DAT and HED, open the archive
    let archive =
        aeco_archive::Archive::open_pair(&source_dat_path, &source_hed_path).map_err(|why| {
            PatchConfigError::OpenArchiveFailed(format!(
                "Couldn't open archive {} + {}: {why:?}",
                source_dat_path.to_string_lossy(),
                source_hed_path.to_string_lossy()
            ))
        })?;

    // Create the directory to which the archive will be unpacked
    std::fs::create_dir(&target_dir_path).map_err(|why| {
        PatchConfigError::CreateTargetDirectoryFailed(format!(
            "Unable to create target directory {}: {}",
            target_dir_path.to_string_lossy(),
            why
        ))
    })?;

    let mut files = Vec::<File>::new();

    // Get files from archive in parallel
    let results: Vec<Result<File, PatchConfigError>> = archive
        .file_names()
        .par_iter()
        .map(|file_name| {
            let file_data = match archive.get_file(file_name) {
                Ok(x) => x,
                Err(why) => {
                    return Err(PatchConfigError::ReadArchiveFailed(format!(
                        "Couldn't read file {file_name} from archive {} + {}: {why:?}",
                        source_dat_path.to_string_lossy(),
                        source_hed_path.to_string_lossy()
                    )));
                }
            };

            let target_file_path = target_dir_path.join(&file_name);

            let file_info = File::new(file_name, &file_data);

            std::fs::write(&target_file_path, &file_data).map_err(|why| {
                PatchConfigError::WriteTargetFileFailed(format!(
                    "Failed to write file {}: {}",
                    target_file_path.to_string_lossy(),
                    why
                ))
            })?;

            Ok(file_info)
        })
        .collect();

    for result in results {
        files.push(result?);
    }

    // // Get files from archive sequentially
    // for file_name in archive.file_names() {
    //     let file_data = match archive.get_file(&file_name) {
    //         Ok(x) => x,
    //         Err(why) => {
    //             return Err(PatchConfigError::ReadArchiveFailed(format!(
    //                 "Couldn't read file {file_name} from archive {} + {}: {why:?}",
    //                 source_dat_path.to_string_lossy(),
    //                 source_hed_path.as_ref().to_string_lossy()
    //             )));
    //         }
    //     };

    //     let target_file_path = target_dir_path.as_ref().join(&file_name);

    //     let file_info = File::new(file_name, &file_data);
    //     files.push(file_info);

    //     if let Err(why) = std::fs::write(&target_file_path, &file_data) {
    //         return Err(PatchConfigError::WriteTargetFileFailed(format!(
    //             "Failed to write file {}: {}",
    //             target_file_path.to_string_lossy(),
    //             why
    //         )));
    //     }
    // }

    let object = FSObject::Archive(Archive {
        name: object_name.to_string(),
        files,
    });

    Ok(Some(object))
}

pub fn process_unpacked_archive<P>(
    source_dir: P,
    target_dir: P,
    object_name: &str,
) -> Result<Archive, PatchConfigError>
where
    P: AsRef<Path>,
{
    let mut files = Vec::<File>::new();

    // Unpacked archives are justdirectories that should be represented as
    // archives
    let dir = process_dir(&source_dir, &target_dir, object_name)?;

    // An unpacked archive should contain files; collect them
    for child in dir.children {
        if let FSObject::File(file) = child {
            files.push(file);
        } else {
            return Err(PatchConfigError::ArchiveContainsDirectory(format!(
                "The archive directory {} does not contain exclusively files",
                source_dir.as_ref().to_string_lossy()
            )));
        }
    }

    Ok(Archive {
        name: object_name.to_string(),
        files,
    })
}
