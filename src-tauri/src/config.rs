use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use tauri::{App, AppHandle, Emitter};

use crate::{AppState, CommandResult, CompletedOnboardingRequest, HasCompletedOnboarding};

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct TtsJobConfig {
    pub output_prefix: String,
    pub output_dir: PathBuf,
    pub start_page: u32,
    /// sets a voice for the narration. see https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/index.html
    pub voice: String,
    pub speed: f32,
    pub combine_pages: bool,
    /// for voices that have multiple speakers, pass a speaker_id.
    pub speaker_id: u32,
    pub use_ocr: bool,
    // end TTS config here
    pub input_file: String,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct TtsAppConfig {
    /// start the narration at a certain page
    pub app_dir: PathBuf,
    pub output_dir: PathBuf,
    pub start_page: u32,
    pub output_prefix: String,
    pub voice: String,
    pub speed: f32,
    pub combine_pages: bool,
    pub speaker_id: u32,
}

// NOTE: APP CONFIG NEEDS TO BE MANAGED MORE CLEANLY BETWEEN UPDATES.
impl TtsAppConfig {
    fn default(output_dir: PathBuf, app_dir: PathBuf) -> Self {
        TtsAppConfig {
            start_page: 0,
            output_dir: output_dir.clone(),
            app_dir,
            voice: String::from("vits-piper-en_US-libritts_r-medium"),
            speed: 1.0,
            combine_pages: true,
            speaker_id: 1,
            output_prefix: String::from("page_"),
        }
    }
    pub fn load_or_default(app_dir: &Path) -> Self {
        let mut config_path = app_dir.to_owned();
        config_path.push("config.toml");

        if fs::exists(&config_path).is_ok_and(|item| item) {
            let config = fs::read_to_string("config.toml").unwrap();
            match toml::from_str(config.as_str()) {
                Ok(config) => config,
                Err(err) => {
                    TtsAppConfig::default(app_dir.to_path_buf(), app_dir.to_path_buf())
                }
            }
        } else {
            let config = TtsAppConfig {
                start_page: 0,
                output_dir: app_dir.to_path_buf(),
                app_dir: app_dir.to_path_buf(),
                voice: String::from("vits-piper-en_US-libritts_r-medium"),
                speed: 1.0,
                combine_pages: true,
                speaker_id: 1,
                output_prefix: String::from("page_"),
            };
            fs::write(config_path, toml::to_string(&config).unwrap()).unwrap();

            config
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_config(state: tauri::State<'_, AppState>) -> CommandResult<TtsAppConfig> {
    // TODO: figure out best practice for returning data based on mutex
    let config: TtsAppConfig = {
        let config = state.settings.lock().await;
        config.clone()
    };
    Ok(config)
}
#[tauri::command]
#[specta::specta]
pub async fn set_config(
    app: AppHandle,
    config: TtsAppConfig,
    state: tauri::State<'_, AppState>,
) -> CommandResult<()> {
    let mut state = state.settings.lock().await;
    *state = config;

    app.emit("settings-changed", "").map_err(|_| anyhow!(""))?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_completed_onboarding(state: tauri::State<'_, AppState>) -> CommandResult<bool> {
    let completed = state.completed_onboarding.lock().await;
    Ok(*completed)
}

#[tauri::command]
#[specta::specta]
pub async fn set_completed_onboarding(
    request: CompletedOnboardingRequest,
    state: tauri::State<'_, AppState>,
    progress_reader: tauri::ipc::Channel<HasCompletedOnboarding>,
) -> CommandResult<()> {
    let mut completed = state.completed_onboarding.lock().await;
    *completed = request.0;
    let _ = progress_reader.send(HasCompletedOnboarding);
    Ok(())
}
