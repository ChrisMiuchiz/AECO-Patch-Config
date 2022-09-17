use std::{fs::read_dir, path::Path};

pub enum TreeError {
    NotDirectory(String),
    ReadDirFailed,
}

pub fn make_tree<P>(path: P) -> Result<Vec<String>, TreeError>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    // The input needs to be a directory and it should exist
    if !path.is_dir() {
        let message = if let Some(path_str) = path.to_str() {
            format!("{path_str} is not a directory.")
        } else {
            "Input path is not a directory".to_string()
        };
        return Err(TreeError::NotDirectory(message));
    }

    let mut result = Vec::<String>::new();

    get_dir_tree(path, &mut result, "")?;

    Ok(result)
}

fn get_dir_tree<P>(path: P, result: &mut Vec<String>, prefix: &str) -> Result<(), TreeError>
where
    P: AsRef<Path>,
{
    let entries = read_dir(path).map_err(|_| TreeError::ReadDirFailed)?;

    for x in entries {
        let child_path = x.map_err(|_| TreeError::ReadDirFailed)?.path();

        let child_name = match child_path.file_name() {
            Some(x) => x.to_string_lossy(),
            None => continue,
        };

        let symbol = if child_path.is_dir() { "📁" } else { "📝" };

        result.push(format!("{prefix}{symbol} {child_name}"));

        if child_path.is_dir() {
            get_dir_tree(child_path, result, &format!("{prefix}  |\t"))?;
        }
    }

    Ok(())
}
