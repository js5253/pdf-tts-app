use crate::{model_manager::SupportedTTSType::Vits, AppState, CommandResult, Error};
use anyhow::anyhow;
use bzip2::{read::BzDecoder, Decompress};
use futures::TryStreamExt;
use image::EncodableLayout;
use rayon::slice::ParallelSlice;
use serde::{Deserialize, Serialize};
use sherpa_onnx::OfflineTtsVitsModelConfig;
use std::{
    io::{self, BufReader, ErrorKind},
    path::{Path, PathBuf},
};
use tar::Archive;
#[tauri::command(async)]
#[specta::specta]
pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    ModelManager::get_models().await
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

#[derive(Deserialize, specta::Type, Serialize, Debug)]
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

pub fn get_model_config(
    base_dir: &Path,
    r#type: SupportedTTSType,
) -> CommandResult<TTSModelConfig> {
    let base_dir = base_dir.to_path_buf();

    let mut model_path: std::path::PathBuf = base_dir.clone();
    model_path.push("en_US-libritts_r-medium.onnx");

    let mut token_path: std::path::PathBuf = base_dir.clone();
    token_path.push("tokens.txt");

    let mut data_dir: std::path::PathBuf = base_dir.clone();
    data_dir.push("tokens.txt");

    match r#type {
        SupportedTTSType::Vits => Ok(TTSModelConfig {
            model: Some(
                model_path
                    .clone()
                    .into_os_string()
                    .into_string()
                    .map_err(|_| anyhow!("can't use model path"))?,
            ),
            tokens: Some(
                token_path
                    .clone()
                    .into_os_string()
                    .into_string()
                    .map_err(|_| anyhow!("can't use token path"))?,
            ),
            data_dir: Some(
                data_dir
                    .clone()
                    .into_os_string()
                    .into_string()
                    .map_err(|_| anyhow!("can't use data path"))?,
            ),
            noise_scale: 0.667,
            noise_scale_w: 0.8,
            length_scale: 1.0,
            dict_dir: None,
            lexicon: None,
            r#type: SupportedTTSType::Vits,
        }),
        SupportedTTSType::Kitten => todo!(),
        SupportedTTSType::Matcha => todo!(),
        SupportedTTSType::Kokoro => todo!(),
        SupportedTTSType::Supertonic => todo!(),
    }
}
impl TryInto<OfflineTtsVitsModelConfig> for TTSModelConfig {
    type Error = crate::Error;

    fn try_into(self) -> Result<OfflineTtsVitsModelConfig, Self::Error> {
        if self.r#type != SupportedTTSType::Vits {
            Err(Error("AASA".into()))
        } else {
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
}

struct ModelManager;
impl ModelManager {
    async fn get_models() -> CommandResult<Vec<Model>> {
        let req: GithubModelResponse = reqwest::get(
            "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models",
        )
        .await
        .map_err(|_| anyhow!("Could not download model list. Check your internet connection. A"))?
        .error_for_status()
        .map_err(|_| anyhow!("Could not download model list. Check your internet connection. B"))?
        .json::<GithubModelResponse>()
        .await
        .map_err(|err| {
            dbg!(err);
            anyhow!("Response was not in expected GithubModelResponse")
        })?;

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
