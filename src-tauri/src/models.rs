use serde::{Deserialize, Serialize};

use crate::{util::CommandResult, AppState, Model};
use anyhow::anyhow;
#[derive(Deserialize, Serialize, specta::Type, Debug, Clone)]
pub struct DownloadManagerProgress {
    name: String,
    progress: f32,
    id: String,
}

#[tauri::command]
#[specta::specta]
pub async fn get_downloaded_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    let data = {
        let models = state
            .downloaded_models
            .lock()
            .await;
        models.to_owned()
    };
    Ok(data)
}

#[tauri::command]
#[specta::specta]
pub async fn set_default_model(
    model_name: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    let models = {
        state.settings.lock().await.voice = model_name;
    };
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn get_download_manager_progress(
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<DownloadManagerProgress>> {
    Ok(state.download_progress.lock().await.clone())
}
