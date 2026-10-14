use std::fs;

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

use crate::{AppState, CommandResult, model_downloader::{self, Model}};

#[derive(Deserialize, Serialize, specta::Type, Debug, Clone)]
pub struct DownloadManagerProgress {
    name: String,
    progress: f32,
    id: String
}

#[tauri::command]
#[specta::specta]
pub async fn get_downloaded_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    Ok(state.downloaded_models.lock().await.clone())
}
#[tauri::command]
#[specta::specta]
pub async fn get_download_manager_progress(state: tauri::State<'_, AppState>) -> CommandResult<Vec<DownloadManagerProgress>>{
    Ok(state.download_progress.lock().await.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn set_default_model(
    model_name: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.settings.lock().await.voice = model_name;
    Ok(())
}