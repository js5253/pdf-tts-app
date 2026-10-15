use crate::{model_manager::SupportedTTSType::Vits, AppState, CommandResult, Error};
use anyhow::{Context, anyhow};
use bzip2::{read::BzDecoder, Decompress};
use futures::TryStreamExt;
use image::EncodableLayout;
use reqwest::{Client, RequestBuilder};
use tracing::{error, info, instrument};
use rayon::slice::ParallelSlice;
use serde::{Deserialize, Serialize};
use sherpa_onnx::{OfflineTtsKokoroModelConfig, OfflineTtsMatchaModelConfig, OfflineTtsVitsModelConfig};
use std::{
    io::{self, BufReader, ErrorKind},
    path::{Path, PathBuf},
};
use tar::Archive;

const SHERPA_URL: &str = "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models";

#[tauri::command(async)]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    ModelManager::get_models(state.gh_access_token).await
}
#[tauri::command(async)]
#[specta::specta]
pub async fn download_model(
    model_id: u32,
    state: tauri::State<'_, AppState>,
) -> CommandResult<(())> {
    let downloaded_model = state.cached_online_models.lock().await;
    let model = downloaded_model
        .iter()
        .find(|item| item.id == model_id)
        .ok_or(anyhow!("Could not find model in list"))?;

    let _ =
        ModelManager::download_model(state.settings.lock().await.app_dir.to_owned(), model).await;

    Ok(())
}

#[derive(Deserialize, specta::Type, Serialize, Debug, Clone)]
pub struct Model {
    r#type: String,
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
#[derive(PartialEq, Eq)]
pub enum SupportedTTSType {
    Vits,
    Kitten,
    Matcha,
    Kokoro,
    Supertonic,
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
                model: Some(model_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                tokens: Some(token_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                data_dir: Some(data_dir.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
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
                model: Some(acoustic_path.into_string().map_err(|_|  anyhow!("Failed to get acoustic path"))?), // or map to acoustic field
                tokens: Some(token_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                data_dir: Some(data_dir.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
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
                model: Some(model_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                tokens: Some(token_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                data_dir: Some(data_dir.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
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
                model: Some(model_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                tokens: Some(token_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                data_dir: Some(data_dir.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
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
                model: Some(model_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                tokens: Some(token_path.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
                data_dir: Some(data_dir.into_string().map_err(|_| anyhow!("Failed to get model path"))?),
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
    async fn get_models(token: &str) -> CommandResult<Vec<Model>> {
        info!("Getting online models w/ token... {}", token);
        let client = Client::new();

        let req: GithubModelResponse = client
        .get(SHERPA_URL)
        .bearer_auth(token)
        .send()
        .await
        .context("Could not download model list")?
        .error_for_status()
        .context("Could not download model list. Check your internet connection. ")?
        .json::<GithubModelResponse>()
        .await
        .context("Failed to deserialize GitHub model response.")?;

        let models = req
            .assets
            .iter()
            .filter(|asset| asset.name.contains("vits_"))
            .map(|item| Model {
                name: item.name.clone(),
                url: item.url.clone(),
                r#type: String::from("vits"),
                id: item.id,
            })
            .collect();
        Ok(models)
    }
    async fn download_model(data_path: PathBuf, model: &Model) -> CommandResult<()> {
        let req = reqwest::blocking::get(model.url.as_str())
            .map_err(|_| anyhow!("Could not download model list"))?;
        let archive = req.bytes().map_err(|_| anyhow!("A"))?;
        let tar = BzDecoder::new(archive.as_bytes());

        let mut decoded = Archive::new(tar);
        decoded
            .unpack(data_path)
            .map_err(|_| anyhow!("Could not unpack downloaded archive."))?;

        Ok(())
    }
}
