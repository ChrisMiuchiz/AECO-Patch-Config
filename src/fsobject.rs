use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum FSObject {
    File(File),
    Directory(Directory),
    Archive(Archive),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub digest: String,
}

impl File {
    pub fn new(name: &str, data: &[u8]) -> Self {
        let digest = md5::compute(&data);
        let digest = format!("{:32x}", digest);

        Self {
            name: name.to_string(),
            digest,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Directory {
    pub name: String,
    pub children: Vec<FSObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Archive {
    pub name: String,
    pub files: Vec<File>,
}
