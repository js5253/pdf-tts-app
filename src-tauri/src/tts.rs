use std::{fs, time::Duration};

use crate::config::TtsJobConfig;
use crate::model_manager::{ModelManager, SupportedTTSType};
use crate::tts_manager::TTSRequest;
use crate::{
    config::{self},
    recents::RecentDoc,
    text_extraction::get_page_contents,
};
use crate::{util::CommandResult, AppState};
use log::info;
use serde::Serialize;
use sherpa_onnx::{GenerationConfig, OfflineTts};
use tokio::time::Instant;
use tracing::instrument;
use uuid::Uuid;

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
