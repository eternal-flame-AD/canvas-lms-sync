use std::{error::Error, path::PathBuf};

use canvas_lms_sync::{
    canvas_api::Client,
    download::Downloader,
    sync::{download_files, download_modules, SyncConfig},
};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    token: String,
    host: String,
    courseid: i64,
    usemodules: bool,
}

impl Into<SyncConfig> for Config {
    fn into(self) -> SyncConfig {
        SyncConfig {
            courseid: self.courseid,
            path: PathBuf::new(),
        }
    }
}

impl Config {
    pub fn read_from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = std::fs::File::open(path)?;
        let config = serde_yaml::from_reader(file)?;
        Ok(config)
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Config::read_from_path("canvas-sync.yml").expect("Failed to read config file");
    let client = Client::new(config.host.clone(), config.token.clone());

    let mut downloader = Downloader::new(reqwest::Client::new(), 4);

    if config.usemodules {
        download_modules(&config.into(), &client, &downloader).await;
    } else {
        download_files(&config.into(), &client, &downloader).await;
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
