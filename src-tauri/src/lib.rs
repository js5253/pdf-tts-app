use anyhow::Context;
use serde::{Deserialize, Serialize};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use std::{
    collections::HashMap,
    env,
    fs::{self},
    path::{PathBuf},
};
use tauri::{Manager};
use tauri_specta::{collect_commands, Builder};
use tokio::sync::{mpsc, Mutex};
use tracing::info;
use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
mod config;
mod download_manager;
mod model_manager;
mod models;
mod recents;
mod text_extraction;
mod tts;
mod util;
use config::{
    get_completed_onboarding, get_config, set_completed_onboarding, set_config, TtsAppConfig,
};
use model_manager::Model;
use models::{
    get_download_manager_progress, get_downloaded_models, set_default_model,
    DownloadManagerProgress,
};
use recents::{get_recent_docs, RecentDoc};
use tts::run_job;

use crate::{
    download_manager::DownloadRequest,
    model_manager::{download_model, get_online_models},
};

struct AppState {
    settings: Mutex<TtsAppConfig>,
    completed_onboarding: Mutex<bool>,
    recent_tts: Mutex<Vec<RecentDoc>>,
    downloaded_models: Arc<Mutex<Vec<Model>>>,
    cached_online_models: Mutex<HashMap<u32, Model>>,
    download_progress: Arc<Mutex<Vec<DownloadManagerProgress>>>,
    gh_access_token: &'static str,
    app_dir: PathBuf,
    download_manager: mpsc::Sender<DownloadRequest>,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let gh_access_token = dotenvy_macro::dotenv!("GH_ACCESS_TOKEN");
    let (mut tx, rx) = mpsc::channel::<DownloadRequest>(300);
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
            get_online_models,
            get_completed_onboarding,
            set_completed_onboarding,
            download_model
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
            let app_dir = app
                .path()
                .app_data_dir()
                .context("Failed to get App Data Dir")?;
            if !app_dir.exists() {
                info!("Creating {:?}", app_dir);
                fs::create_dir_all(&app_dir).context("Failed to create App Dir")?;
            }

            let mut model_path = app_dir.clone();
            model_path.push("tts");
            if !model_path.exists() {
                info!("Creating {:?}", model_path);
                fs::create_dir_all(&model_path).context("Failed to create Model Dir.")?;
            }
            let mut output_path = app_dir.clone();
            output_path.push("out");
            if !output_path.exists() {
                info!("Creating {:?}", output_path);
                fs::create_dir_all(&output_path).context("Failed to create output path.")?;
            }
            let settings = TtsAppConfig::load_or_default(&app_dir);
            let cache_mutex = Mutex::new(HashMap::new());
            let model_arc: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(Vec::new()));
            let value = app_dir.clone();
            let download_handle = Arc::clone(&model_arc);
            let model_mutex = Arc::clone(&model_arc);
            tauri::async_runtime::spawn(async move { download_manager::download_manager(&value, download_handle, rx).await });

            // async_runtime::block_on(async {
            //     let cached = load_cached_models(&mut model_path)
            //         .await
            //         .unwrap_or_default();
            //     let mut models = models.lock().await;
            //     *models = cached.into_values().collect();
            // });
            // let _ = futures::executor::block_on(async {ModelDownloader::load_downloaded_models(Arc::clone(&cache), &model_path.to_path_buf()).await});
            // recheck this to make sure passing app.state is fine
            app.manage(AppState {
                completed_onboarding: Mutex::new(false),
                settings: Mutex::new(settings),
                recent_tts: Mutex::new(Vec::new()),
                cached_online_models: cache_mutex,
                downloaded_models: model_mutex,
                download_progress: Arc::new(Mutex::new(Vec::new())),
                gh_access_token,
                app_dir,
                download_manager: tx
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_online_models,
            set_config,
            get_download_manager_progress,
            get_recent_docs,
            get_completed_onboarding,
            set_completed_onboarding,
            set_default_model,
            get_downloaded_models,
            download_model,
            run_job
        ])
        .run(tauri::generate_context!())
        .expect("Error running app");
    // .map_err(|err| {
    //     error!("{}", err);
    // });
}
