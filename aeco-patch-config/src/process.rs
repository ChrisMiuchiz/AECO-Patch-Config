use crate::error::PatchConfigError;
use crate::fsobject::*;
use rayon::prelude::*;
use std::{
    fs::DirEntry,
    io::Error,
    path::{Path, PathBuf},
};

fn process_dir_entry<P>(
    entry: Result<DirEntry, Error>,
    source_dir: P,
    target_dir: P,
) -> Result<Option<FSObject>, PatchConfigError>
where
    P: AsRef<Path>,
{
    let entry = match entry {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::ReadSourceDirectoryEntryFailed(format!(
                "Couldn't read entry from directory {}: {}",
                source_dir.as_ref().to_string_lossy(),
                why
            )));
        }
    };

    let object_path = entry.path();
    let object_name = match entry.file_name().to_str() {
        Some(x) => x.to_string(),
        None => {
            return Err(PatchConfigError::SourceFileNameInvalid(format!(
                "The object at {} has an invalid name",
                object_path.to_string_lossy()
            )));
        }
    };

    let child = if object_path.is_dir() {
        let mut target_path = PathBuf::new();
        target_path.push(&target_dir);
        target_path.push(&object_name);

        if let Err(why) = std::fs::create_dir(&target_path) {
            return Err(PatchConfigError::CreateTargetDirectoryFailed(format!(
                "Unable to create target subdirectory {}: {}",
                target_path.to_string_lossy(),
                why
            )));
        }

        Some(process_dir(&object_path, &target_path, &object_name)?)
    } else if object_path.is_file() {
        let mut target_path = PathBuf::new();
        target_path.push(&target_dir);
        target_path.push(&object_name);

        process_new_file(&object_path, &target_path, &object_name)?
    } else {
        None
    };

    Ok(child)
}

pub fn process_dir<P>(
    source_dir: P,
    target_dir: P,
    object_name: &str,
) -> Result<FSObject, PatchConfigError>
where
    P: AsRef<Path>,
{
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    let mut children = Vec::<FSObject>::new();

    let readdir = match std::fs::read_dir(&source_dir) {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::ReadSourceDirectoryFailed(format!(
                "Failed to read directory {}: {}",
                source_dir.to_string_lossy(),
                why
            )));
        }
    };

    let results: Vec<Result<Option<FSObject>, PatchConfigError>> = readdir
        .par_bridge()
        .map(|entry| process_dir_entry(entry, &source_dir, &target_dir))
        .collect();

    for result in results {
        if let Some(child) = result? {
            children.push(child)
        }
    }

    // for entry in readdir {
    //     if let Some(child) = process_dir_entry(entry, &source_dir, &target_dir)? {
    //         children.push(child);
    //     }
    // }

    Ok(FSObject::Directory(Directory {
        name: object_name.to_string(),
        children,
    }))
}

fn process_new_archive<P>(
    source_hed_path: P,
    target_dir_path: P,
    object_name: &str,
) -> Result<Option<FSObject>, PatchConfigError>
where
    P: AsRef<Path>,
{
    let mut source_dat_path = source_hed_path.as_ref().to_path_buf();
    source_dat_path.set_extension("dat");

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

    let archive = match aeco_archive::Archive::open_pair(&source_dat_path, &source_hed_path) {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::OpenArchiveFailed(format!(
                "Couldn't open archive {} + {}: {why:?}",
                source_dat_path.to_string_lossy(),
                source_hed_path.to_string_lossy()
            )));
        }
    };

    if let Err(why) = std::fs::create_dir(&target_dir_path) {
        return Err(PatchConfigError::CreateTargetDirectoryFailed(format!(
            "Unable to create target directory {}: {}",
            target_dir_path.to_string_lossy(),
            why
        )));
    }

    let mut files = Vec::<File>::new();

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

            if let Err(why) = std::fs::write(&target_file_path, &file_data) {
                Err(PatchConfigError::WriteTargetFileFailed(format!(
                    "Failed to write file {}: {}",
                    target_file_path.to_string_lossy(),
                    why
                )))
            } else {
                Ok(file_info)
            }
        })
        .collect();

    for result in results {
        files.push(result?);
    }

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

fn process_new_file<P>(
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
        if extension == "dat" {
            return Ok(None);
        } else if extension == "hed" {
            // Make a dir with the extension .archive instead
            if let (Some(target_parent), Some(target_stem)) = (
                target_file_path.as_ref().parent(),
                target_file_path.as_ref().file_stem(),
            ) {
                let mut target_dir = PathBuf::new();
                target_dir.push(target_parent);
                target_dir.push(target_stem);
                target_dir.set_extension("archive");

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

    // println!(
    //     "reading {}, {}",
    //     source_file_path.as_ref().to_string_lossy(),
    //     target_file_path.as_ref().to_string_lossy()
    // );

    let data = match std::fs::read(&source_file_path) {
        Ok(x) => x,
        Err(why) => {
            return Err(PatchConfigError::ReadSourceFileFailed(format!(
                "Failed to read file {}: {}",
                source_file_path.as_ref().to_string_lossy(),
                why
            )));
        }
    };

    let file_info = File::new(object_name, &data);

    if let Err(why) = std::fs::write(&target_file_path, &data) {
        return Err(PatchConfigError::WriteTargetFileFailed(format!(
            "Failed to write file {}: {}",
            target_file_path.as_ref().to_string_lossy(),
            why
        )));
    }
    // println!("Done writing");

    Ok(Some(FSObject::File(file_info)))
}
