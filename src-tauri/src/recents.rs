use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{AppState, util::CommandResult};

#[derive(Serialize, Deserialize, Debug, specta::Type, Clone)]
pub struct RecentDoc {
    pub name: String,
    pub path: String,
    pub id: String
}

#[tauri::command]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_recent_docs(state: tauri::State<'_, AppState>) -> CommandResult<Vec<RecentDoc>> {
        let recents = state.recent_tts.lock().await;
        let recents = recents.clone();
    Ok(recents)
}
