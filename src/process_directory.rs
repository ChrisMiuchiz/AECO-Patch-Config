use crate::{
    constants::*, error::PatchConfigError, fsobject::*, process_archive::*, process_file::*,
};
use rayon::prelude::*;
use std::{
    ffi::OsStr,
    fs::DirEntry,
    io::Error,
    path::{Path, PathBuf},
};

/// If the argument is a valid archive, get its extensionless name
fn archive_directory_stem(path: &Path) -> Option<&OsStr> {
    if path.is_dir() {
        if let (Some(extension), Some(stem)) = (path.extension(), path.file_stem()) {
            if extension == UNPACKED_ARCHIVE_EXTENSION {
                return Some(stem);
            }
        }
    }
    None
}

fn process_dir_entry<P>(
    entry: Result<DirEntry, Error>,
    source_dir: P,
    target_dir: P,
) -> Result<Option<FSObject>, PatchConfigError>
where
    P: AsRef<Path>,
{
    let entry = entry.map_err(|why| {
        PatchConfigError::ReadSourceDirectoryEntryFailed(format!(
            "Couldn't read entry from directory {}: {}",
            source_dir.as_ref().to_string_lossy(),
            why
        ))
    })?;

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

    let child = if let Some(stem) = archive_directory_stem(&object_path) {
        // The object is an unpacked archive.

        let mut target_path = PathBuf::new();
        target_path.push(&target_dir);
        target_path.push(&object_name);

        std::fs::create_dir(&target_path).map_err(|why| {
            PatchConfigError::CreateTargetDirectoryFailed(format!(
                "Unable to create target subdirectory {}: {}",
                target_path.to_string_lossy(),
                why
            ))
        })?;

        Some(FSObject::Archive(process_unpacked_archive(
            &object_path,
            &target_path,
            &stem.to_string_lossy(),
        )?))
    } else if object_path.is_dir() {
        // The object is a regular directory

        let mut target_path = PathBuf::new();
        target_path.push(&target_dir);
        target_path.push(&object_name);

        std::fs::create_dir(&target_path).map_err(|why| {
            PatchConfigError::CreateTargetDirectoryFailed(format!(
                "Unable to create target subdirectory {}: {}",
                target_path.to_string_lossy(),
                why
            ))
        })?;

        Some(FSObject::Directory(process_dir(
            &object_path,
            &target_path,
            &object_name,
        )?))
    } else if object_path.is_file() {
        // The object is a file

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
) -> Result<Directory, PatchConfigError>
where
    P: AsRef<Path>,
{
    let source_dir = source_dir.as_ref();
    let target_dir = target_dir.as_ref();

    let mut children = Vec::<FSObject>::new();

    let readdir = std::fs::read_dir(&source_dir).map_err(|why| {
        PatchConfigError::ReadSourceDirectoryFailed(format!(
            "Failed to read directory {}: {}",
            source_dir.to_string_lossy(),
            why
        ))
    })?;

    // Process dirs in parallel
    let results: Vec<Result<Option<FSObject>, PatchConfigError>> = readdir
        .par_bridge()
        .map(|entry| process_dir_entry(entry, &source_dir, &target_dir))
        .collect();

    for result in results {
        if let Some(child) = result? {
            children.push(child)
        }
    }

    // // Process dirs sequentially
    // for entry in readdir {
    //     if let Some(child) = process_dir_entry(entry, &source_dir, &target_dir)? {
    //         children.push(child);
    //     }
    // }

    Ok(Directory {
        name: object_name.to_string(),
        children,
    })
}
