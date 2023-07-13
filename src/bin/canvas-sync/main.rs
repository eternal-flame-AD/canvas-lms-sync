use std::{collections::HashMap, error::Error, vec};

use canvas_lms_sync::{canvas_api::Client, download::Downloader, File};
use futures::StreamExt;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    token: String,
    host: String,
    courseid: i64,
    usemodules: bool,
}

impl Config {
    pub fn read_from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}

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

async fn download_modules(config: &Config, client: &Client, downloader: &Downloader) {
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

async fn download_files(config: &Config, client: &Client, downloader: &Downloader) {
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

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::read_from_path("canvas-sync.yml").expect("Failed to read config file");
    let client = Client::new(config.host.clone(), config.token.clone());

    let mut downloader = Downloader::new(reqwest::Client::new(), 4);

    if config.usemodules {
        download_modules(&config, &client, &downloader).await;
    } else {
        download_files(&config, &client, &downloader).await;
    };

    info!("Waiting for downloads to finish...");
    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                for progress in downloader.progress().iter() {
                    let progress = progress.lock().unwrap();
                    if let Some(progress) = progress.as_ref() {
                        info!("Progress: {:?}", progress);
                    }
                }
            }
            _ = downloader.finish() => {
                info!("Downloads finished");
                break;
            }
        }
    }
}
