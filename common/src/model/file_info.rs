use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    name: String,
    index: usize,
    content: Vec<u8>,
}

impl FileInfo {
    pub fn new(index: usize, name: String, content: Vec<u8>) -> Self {
        Self {
            index,
            name,
            content,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn content(&self) -> Vec<u8> {
        self.content.clone()
    }
}

impl ToString for FileInfo {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("file deserialization should not fail")
    }
}

impl FromStr for FileInfo {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).map_err(|_| String::from("file deserialization should not fail"))
    }
}
