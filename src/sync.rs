use futures::StreamExt;
use log::{debug, error};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::Mutex;

use crate::{canvas_api::Client, download::Downloader, File};

pub struct IndentStack<T> {
    stack: Vec<(i64, T)>,
}

impl<T> IndentStack<T> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }
    pub fn add(&mut self, indent: i64, item: T) {
        self.stack.retain(|(i, _)| *i < indent);
        self.stack.push((indent, item));
    }
    pub fn get(&self) -> Vec<&T> {
        self.stack.iter().map(|(_, item)| item).collect()
    }
}

pub struct SyncConfig {
    pub courseid: i64,
    pub path: PathBuf,
}

pub async fn download_modules(config: &SyncConfig, client: &Client, downloader: &Downloader) {
    client
        .list_modules(config.courseid)
        .for_each(|module| async {
            let indent = Mutex::new(IndentStack::new());
            match module {
                Ok(module) => {
                    client
                        .list_module_items(config.courseid, module.id)
                        .for_each(|item| async {
                            match item {
                                Ok(item) => {
                                    debug!("Item: {:?}", item);
                                    match item.type_.as_str() {
                                        "SubHeader" => {
                                            indent.lock().await.add(item.indent, item);
                                        }
                                        "File" => {
                                            let file = client
                                                .get_course_file(
                                                    config.courseid,
                                                    item.content_id.expect("No content id"),
                                                )
                                                .await
                                                .unwrap();

                                            let mut file = File::from(file);

                                            file.folder_path =
                                                vec!["Modules".to_string(), module.name.clone()];
                                            file.folder_path.extend(
                                                indent
                                                    .lock()
                                                    .await
                                                    .get()
                                                    .iter()
                                                    .map(|item| item.title.clone()),
                                            );

                                            if file.local_file_matches().unwrap_or(false) {
                                                debug!("File already downloaded: {:?}", file);
                                                return;
                                            }

                                            downloader.submit(file.into());
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) => error!("Failed getting module items: {:?}", e),
                            }
                        })
                        .await;
                }
                Err(e) => error!("Failed getting modules: {:?}", e),
            }
        })
        .await;
}

pub async fn download_files(config: &SyncConfig, client: &Client, downloader: &Downloader) {
    let folders = Mutex::new(HashMap::new());

    client
        .get_all_folders(config.courseid)
        .for_each(|folder| async {
            match folder {
                Ok(folder) => {
                    folders.lock().await.insert(folder.id, folder.clone());
                }
                Err(e) => error!("Failed getting folders: {:?}", e),
            }
        })
        .await;

    let folders = folders.into_inner();

    client
        .get_all_files(config.courseid)
        .for_each(|file| async {
            match file {
                Ok(file) => {
                    let folder_id = file.folder_id;
                    let mut file = File::from(file);
                    file.set_folder_path(&folders, folder_id);

                    if file.local_file_matches().unwrap_or(false) {
                        debug!("File already exists: {:?}", file);
                        return;
                    }

                    downloader.submit(file.into());
                }
                Err(e) => error!("Failed getting files: {:?}", e),
            }
        })
        .await;
}
