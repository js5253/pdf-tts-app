use std::{fs, time::Duration};

use crate::model_manager::{ModelManager, SupportedTTSType};
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
    InProgress(f32),
    Finished,
}

#[instrument(skip(state, progress_reader))]
#[tauri::command(async)]
#[specta::specta]
pub async fn run_job(
    job: config::TtsJobConfig,
    state: tauri::State<'_, AppState>,
    progress_reader: tauri::ipc::Channel<TtsGenerationProgress>,
) -> CommandResult<()> {
    Ok(())
}
