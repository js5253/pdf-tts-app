use anyhow::anyhow;
use serde::{Deserialize, Serialize};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri_plugin_log::{Target, TargetKind};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use std::{env, fmt::Display, fs::{self}, sync::Arc,};
use tauri::{Manager};
use tauri_specta::{collect_commands, Builder};
use tokio::sync::Mutex;

mod config;
mod models;
mod recents;
mod text_extraction;
mod tts;
mod model_manager;
use config::{get_completed_onboarding, get_config, set_completed_onboarding, set_config, TtsAppConfig};
use models::{get_download_manager_progress, get_downloaded_models, set_default_model, DownloadManagerProgress};
use recents::{get_recent_docs, RecentDoc};
use model_manager::{Model};
use tts::run_job;

#[derive(Serialize, Deserialize, Debug, specta::Type)]
struct CompletedOnboardingRequest(bool);

#[derive(Debug, specta::Type)]
pub struct CommandError(pub Error);

impl Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        serializer.serialize_str(&format!("{:?}", self.0))
    }

}
pub type CommandResult<T> = std::result::Result<T, CommandError>;

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<E> From<E> for CommandError
where E: Into<anyhow::Error> {
    fn from(err: E) -> Self {
        Self(err.into().into())
    }
}


#[derive(Debug, specta::Type, Serialize)]
pub struct Error(String);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}
#[derive(Clone, Debug, Serialize, specta::Type)]
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
    settings: Arc<Mutex<TtsAppConfig>>,
    completed_onboarding: Mutex<bool>,
    recent_tts: Mutex<Vec<RecentDoc>>,
    downloaded_models: Arc<Mutex<Vec<Model>>>,
    cached_online_models: Arc<Mutex<Vec<Model>>>,
    download_progress: Mutex<Vec<DownloadManagerProgress>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::registry()
    .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
    .with(fmt::layer())
    .init();

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
        // .plugin(tauri_plugin_fs::init())
        // .plugin(
        //     tauri_plugin_log::Builder::new()
        //         .build(),
        // )
        .setup(move |app| {
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
                let model_path = settings
                    .app_dir
                    .parent()
                    .ok_or(anyhow!("could not open tts model path"))?;
                let cache: Arc<Mutex<Vec<Model>>> = Arc::new(Mutex::new(Vec::new()));
                let models = Arc::new(Mutex::new(Vec::new())); 

                // let _ = futures::executor::block_on(async  {ModelDownloader::load_cached_model_list(Arc::clone(&cache), &model_path.to_path_buf()).await});
                // let _ = futures::executor::block_on(async {ModelDownloader::load_downloaded_models(Arc::clone(&cache), &model_path.to_path_buf()).await});
                // recheck this to make sure passing app.state is fine
                app.manage(AppState {
                    completed_onboarding: Mutex::new(false),
                    settings: Arc::new(Mutex::new(settings)),
                    recent_tts: Mutex::new(Vec::new()),
                    cached_online_models: cache,
                    downloaded_models: models,
                    download_progress: Mutex::new(Vec::new()),
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
        .expect("Error running app");
        // .map_err(|err| {
        //     error!("{}", err);
        // });
}
