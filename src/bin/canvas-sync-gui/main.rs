use std::path::PathBuf;

use canvas_lms_sync::{
    canvas_api::Client,
    sync::{download_files, SyncConfig},
};
use eframe::{
    egui::{CentralPanel, Frame, Margin, RichText, ScrollArea, TopBottomPanel},
    epaint::Vec2,
    App, CreationContext, Theme,
};
use egui_file::FileDialog;
use log::{error, info};
use logger::LogBuffer;
use serde::{Deserialize, Serialize};

mod logger;

pub struct CanvasSyncApp {
    open_file_dialog: Option<FileDialog>,
    input_state: InputState,
    folder_idx: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteConfig {
    token: String,
    host: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum SyncType {
    Modules,
    Files,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CourseConfig {
    course_id: String,
    folder: PathBuf,
    sync_type: SyncType,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct InputState {
    remotes: Vec<RemoteConfig>,
    courses: Vec<CourseConfig>,
    selected_remote: Option<usize>,
    selected_course: Option<usize>,
}

impl CanvasSyncApp {
    fn new(cc: &CreationContext) -> Self {
        let mut state = InputState::default();
        if let Some(storage) = cc.storage {
            if let Some(data) = storage.get_string("input_state") {
                match serde_json::from_str(&data) {
                    Ok(input_state) => state = input_state,
                    Err(e) => error!("Failed to deserialize input state: {}", e),
                }
            }
        }
        Self {
            folder_idx: 0,
            open_file_dialog: None,
            input_state: state,
        }
    }
}

pub async fn sync(remote: &RemoteConfig, course: &CourseConfig) {
    info!("Syncing course {}...", course.course_id);
    let client = Client::new(remote.host.clone(), remote.token.clone());
    let mut downloader = canvas_lms_sync::download::Downloader::new(reqwest::Client::new(), 4);
    match course.sync_type {
        SyncType::Files => {
            download_files(
                &SyncConfig {
                    courseid: course.course_id.parse().unwrap(),
                    path: course.folder.clone(),
                },
                &client,
                &downloader,
            )
            .await
        }
        SyncType::Modules => {
            canvas_lms_sync::sync::download_modules(
                &SyncConfig {
                    courseid: course.course_id.parse().unwrap(),
                    path: course.folder.clone(),
                },
                &client,
                &downloader,
            )
            .await
        }
    }
    info!("Waiting for all downloads to finish...");
    downloader.finish().await;
    info!("Done!");
}

impl App for CanvasSyncApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(data) = serde_json::to_string(&self.input_state) {
            storage.set_string("input_state", data);
        }
    }
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Canvas Sync");
        });
        CentralPanel::default().show(ctx, |ui| {
            if let Some(dialog) = &mut self.open_file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(path) = dialog.path() {
                        self.input_state.courses[self.folder_idx].folder = path;
                    }
                }
            }

            ui.separator();

            ui.heading("Remotes");
            let mut remove_remote = None;
            for (i, remote) in self.input_state.remotes.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui
                        .radio(
                            if let Some(s) = self.input_state.selected_remote {
                                s == i
                            } else {
                                false
                            },
                            "",
                        )
                        .clicked()
                    {
                        self.input_state.selected_remote = Some(i);
                    }
                    if ui.button("Remove").clicked() {
                        remove_remote = Some(i);
                    }
                    ui.label("Host");
                    ui.text_edit_singleline(&mut remote.host);
                    ui.label("Token");
                    ui.text_edit_singleline(&mut remote.token);
                });
            }
            if let Some(i) = remove_remote {
                self.input_state.remotes.remove(i);
            }

            if ui.button("Add Remote").clicked() {
                self.input_state.remotes.push(RemoteConfig {
                    token: String::new(),
                    host: "https://canvas.instructure.com".to_string(),
                });
            }

            ui.separator();

            ui.heading("Courses");
            let mut remove_course = None;
            for (i, course) in self.input_state.courses.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    if ui
                        .radio(
                            if let Some(s) = self.input_state.selected_course {
                                s == i
                            } else {
                                false
                            },
                            "",
                        )
                        .clicked()
                    {
                        self.input_state.selected_course = Some(i);
                    }
                    if ui.button("Remove").clicked() {
                        remove_course = Some(i);
                    }
                    ui.label("Course ID");
                    ui.text_edit_singleline(&mut course.course_id);
                    ui.label("Sync Type");
                    ui.radio_value(&mut course.sync_type, SyncType::Files, "Files");
                    ui.radio_value(&mut course.sync_type, SyncType::Modules, "Modules");
                    ui.label("Folder");
                    if ui.button("Select Folder").clicked() {
                        let mut dialog = FileDialog::select_folder(None);
                        dialog.open();
                        self.folder_idx = i;
                        self.open_file_dialog = Some(dialog);
                    }
                    ui.label(course.folder.to_string_lossy());
                });
            }
            if let Some(i) = remove_course {
                self.input_state.courses.remove(i);
            }

            if ui.button("Add Course").clicked() {
                self.input_state.courses.push(CourseConfig {
                    course_id: String::new(),
                    folder: PathBuf::new(),
                    sync_type: SyncType::Files,
                });
            }

            ui.separator();

            if ui.button("Sync").clicked() {
                let remote = self
                    .input_state
                    .remotes
                    .get(self.input_state.selected_remote.unwrap())
                    .unwrap()
                    .clone();
                let course = self
                    .input_state
                    .courses
                    .get(self.input_state.selected_course.unwrap())
                    .unwrap()
                    .clone();
                tokio::spawn(async move { sync(&remote, &course).await });
            }

            ui.separator();

            ui.heading("Logs");
            ScrollArea::vertical().show(ui, |ui| {
                Frame::none()
                    .inner_margin(Margin {
                        left: 10.0,
                        right: 10.0,
                        top: 10.0,
                        bottom: 10.0,
                    })
                    .show(ui, |ui| unsafe {
                        for log in LOG_BUFFER.logs.lock().unwrap().iter() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("[{}] {}", log.level, log.message)));
                            });
                        }
                    });
            });
        });
    }
}

static mut LOG_BUFFER: LogBuffer = LogBuffer::new();

#[tokio::main]
async fn main() {
    unsafe {
        log::set_max_level(log::LevelFilter::Info);
        log::set_logger(&LOG_BUFFER).unwrap();
    }

    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = false;
    native_options.default_theme = Theme::Light;
    native_options.initial_window_size = Some(Vec2::new(800.0, 600.0));

    info!("Starting Canvas Sync GUI");

    eframe::run_native(
        "Canvas Sync",
        native_options,
        Box::new(|cc| Box::new(CanvasSyncApp::new(cc))),
    )
    .expect("Failed to run gui");
}
