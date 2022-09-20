use std::cmp::Ordering;
use std::{fs::read_dir, path::Path};

use eframe::egui;
use eframe::epaint::Vec2;

pub enum TreeError {
    NotDirectory(String),
    ReadDirFailed,
}

enum FileTreeNode {
    Directory(FileTree),
    File(String),
}

impl FileTreeNode {
    pub fn name(&self) -> &str {
        match &self {
            FileTreeNode::Directory(tree) => &tree.name,
            FileTreeNode::File(name) => name,
        }
    }
}

pub struct FileTree {
    name: String,
    nodes: Vec<FileTreeNode>,
    header_size: Option<Vec2>,
    body_size: Option<Vec2>,
}

impl FileTree {
    pub fn new<P>(path: P) -> Result<FileTree, TreeError>
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

        let entries = read_dir(path).map_err(|_| TreeError::ReadDirFailed)?;

        // Gather all sub-paths into nodes
        let mut nodes = Vec::<FileTreeNode>::new();

        for x in entries {
            let child_path = x.map_err(|_| TreeError::ReadDirFailed)?.path();

            let child_name = match child_path.file_name() {
                Some(x) => x.to_string_lossy(),
                None => continue,
            };

            let new_node = if child_path.is_dir() {
                FileTreeNode::Directory(Self::new(child_path)?)
            } else {
                FileTreeNode::File(child_name.to_string())
            };

            nodes.push(new_node);
        }

        // The top-level header name
        let root_name = if let Some(name_os_str) = path.file_name() {
            name_os_str.to_string_lossy().to_string()
        } else {
            "Files".to_string()
        };

        // Sort the nodes so they are arranged with directories first, then
        // alphabetically
        nodes.sort_by(|x, y| {
            if let (FileTreeNode::Directory(_), FileTreeNode::File(_)) = (x, y) {
                return Ordering::Less;
            }

            if let (FileTreeNode::Directory(_), FileTreeNode::File(_)) = (y, x) {
                return Ordering::Greater;
            }

            x.name().cmp(y.name())
        });

        Ok(Self {
            name: root_name,
            nodes,
            body_size: None,
            header_size: None,
        })
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let x = egui::CollapsingHeader::new(&self.name)
            .default_open(true)
            .show(ui, |ui| {
                for node in &mut self.nodes {
                    match node {
                        FileTreeNode::Directory(d) => {
                            d.show(ui);
                        }
                        FileTreeNode::File(name) => {
                            ui.label(name.as_str());
                        }
                    }
                }
            });

        self.header_size = Some(x.header_response.rect.size());

        self.body_size = if let Some(body_response) = x.body_response {
            Some(body_response.rect.size())
        } else {
            None
        };
    }

    pub fn height(&self) -> f32 {
        let mut accumulator = 0f32;

        if let Some(header_size) = self.header_size {
            accumulator += header_size.y;
        }

        if let Some(body_size) = self.body_size {
            accumulator += body_size.y;
        }

        accumulator
    }
}
