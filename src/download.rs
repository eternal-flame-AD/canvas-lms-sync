use crate::defer;
use crossbeam_channel::bounded;
use log::error;
use std::{
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::task::JoinSet;

pub struct Downloader {
    task_channel: Option<crossbeam::channel::Sender<DownloadTask>>,
    joinset: JoinSet<()>,
    progress: Arc<Vec<Mutex<Option<DownloadProgress>>>>,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub total: usize,
    pub downloaded: usize,
    pub task: DownloadTask,
}

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum DownloadError {
    Io(std::io::Error),
    Reqwest(reqwest::Error),
}

pub async fn download_file(
    client: reqwest::Client,
    url: &str,
    path: &PathBuf,
    progress: &Mutex<Option<DownloadProgress>>,
) -> Result<(), DownloadError> {
    defer!({
        progress.lock().unwrap().take();
    });

    let mut resp = client
        .get(url)
        .send()
        .await
        .map_err(DownloadError::Reqwest)?;

    progress.lock().unwrap().replace(DownloadProgress {
        total: resp.content_length().unwrap_or(0) as usize,
        downloaded: 0,
        task: DownloadTask {
            url: url.to_string(),
            path: path.clone(),
        },
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(DownloadError::Io)?;
    }
    let mut file = std::fs::File::create(&path).map_err(DownloadError::Io)?;
    while let Some(chunk) = resp.chunk().await.map_err(DownloadError::Reqwest)? {
        progress.lock().unwrap().as_mut().unwrap().downloaded += chunk.len();
        file.write_all(&chunk).map_err(DownloadError::Io)?;
    }
    Ok(())
}

impl Downloader {
    pub fn new(client: reqwest::Client, nprocs: usize) -> Self {
        let (tx, rx) = bounded::<DownloadTask>(100);
        let reqwest = client.clone();
        let mut js = JoinSet::new();

        let mut progress = Vec::new();
        for _ in 0..nprocs {
            progress.push(Mutex::new(None));
        }
        let progress = Arc::new(progress);

        for id in 0..nprocs {
            let rx = rx.clone();
            let reqwest = reqwest.clone();
            let progress = progress.clone();
            js.spawn(async move {
                for task in rx {
                    match download_file(reqwest.clone(), &task.url, &task.path, &progress[id]).await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Failed to download file: {:?}", e);
                        }
                    }
                }
            });
        }
        Self {
            task_channel: Some(tx),
            joinset: js,
            progress,
        }
    }
    pub fn progress(&self) -> Arc<Vec<Mutex<Option<DownloadProgress>>>> {
        self.progress.clone()
    }
    pub fn submit(&self, task: DownloadTask) {
        self.task_channel
            .as_ref()
            .expect("attempt to submit task to closed downloader")
            .send(task)
            .unwrap();
    }
    pub async fn finish(&mut self) {
        self.task_channel = None;
        while let Some(_) = self.joinset.join_next().await {}
    }
}
