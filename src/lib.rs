use std::collections::HashMap;

use canvas_api::files::{FileResp, FolderResp};
use download::DownloadTask;
use path::sanitize_file_name;

pub mod canvas_api;
mod defer;
pub mod download;
mod path;

#[derive(Debug)]
pub struct File {
    pub folder_path: Vec<String>,
    pub file_name: String,
    pub size: i64,
    pub created_at: String,
    pub updated_at: String,
    pub modified_at: String,
    pub url: String,
}

impl Into<DownloadTask> for File {
    fn into(self) -> DownloadTask {
        DownloadTask {
            path: self.local_path(),
            url: self.url,
        }
    }
}

impl File {
    pub fn set_folder_path(&mut self, folder_map: &HashMap<i64, FolderResp>, folder_id: i64) {
        self.folder_path.clear();
        let mut cur_folder = folder_id;
        while cur_folder != 0 {
            let folder = folder_map.get(&cur_folder).unwrap();
            self.folder_path.push(folder.name.clone());
            cur_folder = folder.parent_folder_id.unwrap_or(0);
        }
        self.folder_path.reverse();
    }
    pub fn local_file_matches(&self) -> Result<bool, std::io::Error> {
        let path = self.local_path();
        if !path.exists() {
            return Ok(false);
        }
        let metadata = path.metadata()?;
        if metadata.len() != self.size as u64 {
            return Ok(false);
        }
        Ok(true)
    }
    pub fn local_path(&self) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::new();
        for folder in self.sanitized_folder_path() {
            path.push(folder);
        }
        path.push(self.sanitized_file_name());
        path
    }
    pub fn sanitized_folder_path(&self) -> Vec<String> {
        self.folder_path
            .iter()
            .map(|s| sanitize_file_name(s))
            .collect::<Vec<_>>()
    }
    pub fn sanitized_file_name(&self) -> String {
        sanitize_file_name(&self.file_name)
    }
}

impl From<FileResp> for File {
    fn from(value: FileResp) -> Self {
        Self {
            folder_path: Vec::new(),
            file_name: value.display_name,
            size: value.size,
            created_at: value.created_at,
            updated_at: value.updated_at,
            modified_at: value.modified_at,
            url: value.url,
        }
    }
}
