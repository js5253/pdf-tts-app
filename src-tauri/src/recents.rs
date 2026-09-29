use serde::{Deserialize, Serialize};

use crate::{AppState, CommandResult};

#[derive(Serialize, Deserialize, Debug, specta::Type, Clone)]
pub struct RecentDoc {
    pub name: String,
    pub path: String,
    pub id: String
}

#[tauri::command]
#[specta::specta]
pub async fn get_recent_docs(state: tauri::State<'_, AppState>) -> CommandResult<Vec<RecentDoc>> {
    // TODO: figure out best practice for returning data based on mutex
        let recents = state.recent_tts.lock().await;
        let recents = recents.clone();
    Ok(recents)
}
