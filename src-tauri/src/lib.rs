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

use tokio::{sync::Mutex};

use crate::config::TtsAppConfig;

mod tts;
mod text_extraction;
mod config;
use tts::run_job;
use config::{get_downloaded_models, set_default_model, get_completed_onboarding, set_completed_onboarding, get_config};
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

struct AppState {
    settings: Mutex<TtsAppConfig>,
    completed_onboarding: Mutex<bool>,
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            set_default_model,
            get_config,
            get_downloaded_models,
            run_job,
            get_completed_onboarding,
            set_completed_onboarding
        ]);
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .setup(move |app| {
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
                app.manage(AppState {
                    completed_onboarding: Mutex::new(false),
                    settings: Mutex::new(TtsAppConfig::load_or_default(&app_dir)),
                });
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_completed_onboarding,
            set_completed_onboarding,
            set_default_model,
            get_downloaded_models,
            run_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
