use std::fs;

use anyhow::{Context, anyhow};

use crate::{AppState, CommandResult};

#[tauri::command]
#[specta::specta]
pub async fn get_downloaded_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<String>> {

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