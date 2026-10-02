use std::fs;

use anyhow::{Context, anyhow};

use crate::{AppState, CommandResult};

#[tauri::command]
#[specta::specta]
pub async fn get_downloaded_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<String>> {
    let mut models: Vec<String> = Vec::new();
    let binding = state.settings.lock().await;
        let model_path = binding.app_dir
        .parent()
        .ok_or(anyhow!("could not open tts model path"))?;
    dbg!(model_path);

    let dir = fs::read_dir(model_path.join("tts")).context("No TTS Model")?;
    dir.for_each(|item| models.push(item.unwrap().file_name().to_string_lossy().to_string()));

    Ok(models)
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