
use crate::tts_manager::TTSRequest;
use crate::config::{self};
use crate::{util::CommandResult, AppState};
use serde::Serialize;
use tracing::instrument;

#[derive(Clone, Debug, Serialize, specta::Type)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum TtsGenerationProgress {
    InProgress(f32, Option<Vec<f32>>),
    Finished,
}

#[instrument(skip(state))]
#[tauri::command(async)]
#[specta::specta]
pub async fn run_job(
    job: config::TtsJobConfig,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    state.tts_manager.send(job).await?;
    Ok(())
}
#[instrument(skip(state))]
#[specta::specta]
#[tauri::command]
pub async fn get_tts_progress(state: tauri::State<'_, AppState>,
) -> CommandResult<Vec<TTSRequest>>{
    Ok((*state.tts_progress.lock().await.clone()).to_vec())
}
