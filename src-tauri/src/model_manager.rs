use crate::{AppState, CommandResult, Error};
use anyhow::{anyhow, Context};
use bzip2::read::BzDecoder;
use futures::TryStreamExt;
use image::EncodableLayout;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sherpa_onnx::{
    OfflineTtsKokoroModelConfig, OfflineTtsMatchaModelConfig, OfflineTtsVitsModelConfig,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tar::Archive;
use tokio::sync::Mutex;
use tracing::{info, instrument};

const SHERPA_URL: &str = "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models";

#[tauri::command(async)]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    ModelManager::get_models(
        state.gh_access_token,
        false,
        Arc::clone(&state.cached_online_models),
    )
    .await
}
#[tauri::command(async)]
#[specta::specta]
#[instrument(skip(state))]
pub async fn download_model(model_id: u32, state: tauri::State<'_, AppState>) -> CommandResult<()> {
    let downloaded_model = state.cached_online_models.lock().await;
    let model = downloaded_model
        .get(&model_id)
        .ok_or(anyhow!("Could not find model in list"))?;
    info!("Requested model download {:?}", model);
    let _ = ModelManager::download_model(
        state.gh_access_token,
        state.settings.lock().await.app_dir.to_owned(),
        model,
    )
    .await;

    Ok(())
}

#[derive(Deserialize, specta::Type, Serialize, Debug, Clone)]
pub struct Model {
    r#type: SupportedTTSType,
    name: String,
    url: String,
    id: u32,
}
#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    url: String,
    browser_download_url: String,
    id: u32,
}
#[derive(Deserialize)]
struct GithubModelResponse {
    assets: Vec<GithubAsset>,
}
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone, specta::Type)]
pub enum SupportedTTSType {
    Vits,
    Kitten,
    Matcha,
    Kokoro,
    Supertonic,
}
fn get_type_from_string(file_name: &str) -> Option<SupportedTTSType> {
    if file_name.contains("vits") {
        Some(SupportedTTSType::Vits)
    } else if file_name.contains("kitten") {
        Some(SupportedTTSType::Kitten)
    } else if file_name.contains("matcha") {
        Some(SupportedTTSType::Matcha)
    } else if file_name.contains("kokoro") {
        Some(SupportedTTSType::Kokoro)
    } else if file_name.contains("supertonic") {
        Some(SupportedTTSType::Supertonic)
    } else {
        None
    }
}

pub struct TTSModelConfig {
    pub model: Option<String>,
    pub lexicon: Option<String>,
    pub tokens: Option<String>,
    pub data_dir: Option<String>,
    pub noise_scale: f32,
    pub noise_scale_w: f32,
    pub length_scale: f32,
    pub dict_dir: Option<String>,
    pub r#type: SupportedTTSType,
}

// let mut model_path: std::path::PathBuf = current_path.clone();
// model_path.push("en_US-libritts_r-medium.onnx");

// let mut token_path: std::path::PathBuf = current_path.clone();
// token_path.push("tokens.txt");

// let mut data_dir = current_path.clone();
// data_dir.push("espeak-ng-data");
impl TryInto<OfflineTtsVitsModelConfig> for TTSModelConfig {
    type Error = crate::Error;

    fn try_into(self) -> Result<OfflineTtsVitsModelConfig, Self::Error> {
        if self.r#type != SupportedTTSType::Vits {
            return Err(Error("Invalid TTS type for VITS config".into()));
        }
        Ok(OfflineTtsVitsModelConfig {
            model: self.model,
            lexicon: self.lexicon,
            tokens: self.tokens,
            data_dir: self.data_dir,
            noise_scale: self.noise_scale,
            noise_scale_w: self.noise_scale_w,
            length_scale: self.length_scale,
            dict_dir: self.dict_dir,
            ..Default::default()
        })
    }
}

pub fn get_model_config(
    base_dir: &Path,
    r#type: SupportedTTSType,
) -> CommandResult<TTSModelConfig> {
    let base_dir = base_dir.to_path_buf();

    let mut token_path = base_dir.clone();
    token_path.push("tokens.txt");

    let mut data_dir = base_dir.clone();
    data_dir.push("espeak-ng-data");

    match r#type {
        SupportedTTSType::Vits => {
            let mut model_path = base_dir.clone();
            model_path.push("en_US-libritts_r-medium.onnx");
            Ok(TTSModelConfig {
                model: Some(
                    model_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                tokens: Some(
                    token_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                data_dir: Some(
                    data_dir
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                dict_dir: None,
                lexicon: None,
                r#type: SupportedTTSType::Vits,
            })
        }
        SupportedTTSType::Matcha => {
            // Matcha requires an acoustic model and a vocoder (e.g., Vocos)
            let mut acoustic_path = base_dir.clone();
            acoustic_path.push("model.onnx");
            let mut vocoder_path = base_dir.clone();
            vocoder_path.push("vocos.onnx");

            Ok(TTSModelConfig {
                model: Some(
                    acoustic_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get acoustic path"))?,
                ), // or map to acoustic field
                tokens: Some(
                    token_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                data_dir: Some(
                    data_dir
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                dict_dir: None,
                lexicon: None,
                r#type: SupportedTTSType::Matcha,
            })
        }
        SupportedTTSType::Kokoro => {
            let mut model_path = base_dir.clone();
            model_path.push("model.onnx");
            Ok(TTSModelConfig {
                model: Some(
                    model_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                tokens: Some(
                    token_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                data_dir: Some(
                    data_dir
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                dict_dir: None,
                lexicon: None,
                r#type: SupportedTTSType::Kokoro,
            })
        }
        SupportedTTSType::Kitten => {
            let mut model_path = base_dir.clone();
            model_path.push("model.onnx");
            Ok(TTSModelConfig {
                model: Some(
                    model_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                tokens: Some(
                    token_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                data_dir: Some(
                    data_dir
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                dict_dir: None,
                lexicon: None,
                r#type: SupportedTTSType::Kitten,
            })
        }
        SupportedTTSType::Supertonic => {
            let mut model_path = base_dir.clone();
            model_path.push("model.onnx");
            Ok(TTSModelConfig {
                model: Some(
                    model_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                tokens: Some(
                    token_path
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                data_dir: Some(
                    data_dir
                        .into_string()
                        .map_err(|_| anyhow!("Failed to get model path"))?,
                ),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                dict_dir: None,
                lexicon: None,
                r#type: SupportedTTSType::Supertonic,
            })
        }
    }
}

impl TryInto<OfflineTtsMatchaModelConfig> for TTSModelConfig {
    type Error = crate::Error;
    fn try_into(self) -> Result<OfflineTtsMatchaModelConfig, Self::Error> {
        if self.r#type != SupportedTTSType::Matcha {
            return Err(Error("Expected Matcha TTS type".into()));
        }
        Ok(OfflineTtsMatchaModelConfig {
            acoustic_model: self.model,
            tokens: self.tokens,
            data_dir: self.data_dir,
            lexicon: self.lexicon,
            dict_dir: self.dict_dir,
            ..Default::default()
        })
    }
}

impl TryInto<OfflineTtsKokoroModelConfig> for TTSModelConfig {
    type Error = crate::Error;
    fn try_into(self) -> Result<OfflineTtsKokoroModelConfig, Self::Error> {
        if self.r#type != SupportedTTSType::Kokoro {
            return Err(Error("Expected Kokoro TTS type".into()));
        }
        Ok(OfflineTtsKokoroModelConfig {
            model: self.model,
            tokens: self.tokens,
            data_dir: self.data_dir,
            lexicon: self.lexicon,
            ..Default::default()
        })
    }
}

pub struct ModelManager;
impl ModelManager {
    #[instrument]
    async fn get_models(
        token: &str,
        force_recheck: bool,
        cached_online_models: Arc<Mutex<HashMap<u32, Model>>>,
    ) -> CommandResult<Vec<Model>> {
        let mut cached_online_models = cached_online_models.lock().await;
        if !cached_online_models.is_empty() && !force_recheck {
            info!("Using cached online models");
            let arr: Vec<Model> = cached_online_models.clone().into_values().collect();
            Ok(arr)
        } else {
            info!("Getting online models w/ token... {}", token);
            let client = Client::new();

            let req: GithubModelResponse = client
                .get(SHERPA_URL)
                .header("User-Agent", "js5253-pdf-tts-app")
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| {
                    info!("{}", e);
                    anyhow!("Could not download model list.")
                })?
                // .inspect( async |body| {
                //     let response: String = body.text(),await;
                //     info!("Request response: {}", response);
                // })?
                .error_for_status()
                .map_err(|e| {
                    info!("Error: {}", e);
                    anyhow!("Could not download model list.")
                })?
                .json::<GithubModelResponse>()
                .await
                .context("Failed to deserialize GitHub model response.")?;

            let models: HashMap<u32, Model> = req
                .assets
                .iter()
                .filter_map(|item| {
                    let r#type = get_type_from_string(item.name.as_str());

                    r#type.as_ref()?;
                    let r#type = r#type.unwrap();
                    Some((
                        item.id,
                        Model {
                            name: item.name.clone(),
                            url: item.url.clone(),
                            id: item.id,
                            r#type,
                        },
                    ))
                })
                .collect();
            info!("Models: {:?}", models);
            *cached_online_models = models.clone();
            let arr: Vec<Model> = models.into_values().collect();

            Ok(arr)
        }
    }
    async fn download_model(token: &str, data_path: PathBuf, model: &Model) -> CommandResult<()> {
        
        // tokio::task::spawn_blocking(move || {
        // let client = reqwest::blocking::Client::new();
        // let req = client
        //     .get(url)
        //     .header("User-Agent", "js5253-pdf-tts-app")
        //     .bearer_auth(token)
        //     .send()
        //     .map_err(|_| anyhow!("Could not download model")).expect("msg");
        // let archive = req.bytes().map_err(|_| anyhow!("A")).unwrap();
        // let tar = BzDecoder::new(archive.as_bytes());

        // let mut decoded = Archive::new(tar);
        // decoded
        //     .unpack(data_path)
        //     .map_err(|_| anyhow!("Could not unpack downloaded archive.")).unwrap();


        // });
        Ok(())
    }
}
