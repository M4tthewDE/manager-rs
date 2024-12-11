use anyhow::Result;
use std::{fs::DirEntry, path::PathBuf};

#[derive(Default, Debug)]
pub struct ComposeFile {
    pub name: PathBuf,
    pub content: String,
}

impl ComposeFile {
    pub fn new(entry: DirEntry) -> Result<Self> {
        Ok(Self {
            name: entry.path(),
            content: std::fs::read_to_string(entry.path())?,
        })
    }
}
