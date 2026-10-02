use std::{
    io::{self, BufReader, ErrorKind},
    path::PathBuf,
};

use crate::{AppState, CommandResult};
use anyhow::anyhow;
use bzip2::{read::BzDecoder, Decompress};
use futures::TryStreamExt;
use image::EncodableLayout;
use rayon::slice::ParallelSlice;
use serde::{Deserialize, Serialize};
use tar::Archive;
#[tauri::command(async)]
#[specta::specta]
pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    ModelDownloader::get_models().await
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

    let _ = ModelDownloader::download_model(state.settings.lock().await.app_dir.to_owned(), model)
        .await;

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
struct ModelDownloader;
impl ModelDownloader {
    async fn get_models() -> CommandResult<Vec<Model>> {
        let req: GithubModelResponse = reqwest::get(
            "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models",
        )
        .await
        .map_err(|_| anyhow!("Could not download model list. Check your internet connection."))?
        .error_for_status()
        .map_err(|_| anyhow!("Could not download model list. Check your internet connection."))?
        .json::<GithubModelResponse>()
        .await
        .map_err(|_| anyhow!("Response was not in expected GithubModelResponse"))?;

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
