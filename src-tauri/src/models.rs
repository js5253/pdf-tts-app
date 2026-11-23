use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{util::CommandResult, AppState, Model};
#[derive(Deserialize, Serialize, specta::Type, Debug, Clone)]
pub struct DownloadManagerProgress {
    pub name: String,
    pub progress: usize,
    pub id: u32,
    pub r#type: String
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_downloaded_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    let data = {
        let models = state
            .downloaded_models
            .lock()
            .await;
        models.to_owned()
    };
    let arr = data.values().map(|item| item.to_owned()).collect();
    Ok(arr)
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn set_default_model(
    model_name: String,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    {
        state.settings.lock().await.voice = model_name;
    };
    Ok(())
}
#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_download_manager_progress(
    state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<DownloadManagerProgress>> {
    Ok(state.download_progress.lock().await.clone())
}
