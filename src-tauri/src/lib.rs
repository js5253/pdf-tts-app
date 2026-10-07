use anyhow::anyhow;
use log::LevelFilter;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::{Deserialize, Serialize};

#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use std::{
    env,
    fs::{self},
};
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

use tokio::sync::Mutex;

mod commands;
mod model_downloader;
mod tts;
use commands::config::{get_completed_onboarding, get_config, set_completed_onboarding, set_config};
use commands::models::{get_downloaded_models, set_default_model, get_download_manager_progress};
use commands::recents::{get_recent_docs, RecentDoc};
use tts::run_job;

use crate::{commands::{config::TtsAppConfig, models::{self, DownloadManagerProgress}}, model_downloader::{Model, ModelDownloader, get_models}};
#[derive(Serialize, Deserialize, Debug, specta::Type)]

struct CompletedOnboardingRequest(bool);

pub type CommandResult<T> = std::result::Result<T, Error>;

#[derive(Debug, specta::Type, Serialize)]
pub struct Error(String);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}
#[derive(Clone, Serialize, specta::Type)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
enum TtsGenerationProgress {
    InProgress(f32),
    Finished,
}
#[derive(Clone, Default, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
struct HasCompletedOnboarding;

struct AppState {
    settings: Mutex<TtsAppConfig>,
    completed_onboarding: Mutex<bool>,
    recent_tts: Mutex<Vec<RecentDoc>>,
    downloaded_models: Mutex<Vec<Model>>,
    cached_online_models: Mutex<Vec<Model>>,
    download_progress: Mutex<Vec<DownloadManagerProgress>>
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
            let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            get_download_manager_progress,
            set_default_model,
            get_config,
            set_config,
            get_recent_docs,
            get_downloaded_models,
            run_job,
            // get_online_models,
            get_completed_onboarding,
            set_completed_onboarding
        ]);
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Warn)
                .build(),
        )
        .setup(async move |app| {
            builder.mount_events(app);
            if let Ok(app_dir) = app.path().app_data_dir() {
                if !app_dir.exists() {
                    fs::create_dir_all(&app_dir).expect("Failed to create App Dir");

                    let mut model_path = app_dir.clone();
                    model_path.push("tts");
                    if !model_path.exists() {
                        fs::create_dir_all(&model_path).expect("Failed to create Model Dir.");
                    }
                    let mut output_path = app_dir.clone();
                    output_path.push("tts");
                    if !output_path.exists() {
                        fs::create_dir_all(&output_path).expect("Failed to create output path.");
                    }
                }
                let settings = TtsAppConfig::load_or_default(&app_dir);
                let mut models: Vec<Model> = Vec::new();
                let model_path = settings
                    .app_dir
                    .parent()
                    .ok_or(anyhow!("could not open tts model path"))?;
                // recheck this to make sure passing app.state is fine
                let cache = Mutex::new(Vec::new());
                let models = ModelDownloader::get_models(&cache).await;
                app.manage(AppState {
                    completed_onboarding: Mutex::new(false),
                    settings: Mutex::new(settings),
                    recent_tts: Mutex::new(Vec::new()),
                    cached_online_models: Mutex::new(Vec::new()),
                    downloaded_models: Mutex::new(models),
                    download_progress: Mutex::new(Vec::new())
                });
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            // get_online_models,
            set_config,
            get_recent_docs,
            get_completed_onboarding,
            set_completed_onboarding,
            set_default_model,
            get_downloaded_models,
            run_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
 
}
