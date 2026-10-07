use std::{
    fs,
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
use tokio::sync::Mutex;

const SHERPA_DOWNLOAD_URL: &str = "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models";

// #[tauri::command(async)]
// #[specta::specta]
// pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
//     ModelDownloader::get_models(state.cached_online_models).await
// }#[tauri::command(async)]
// #[specta::specta]
// pub async fn download_model(
//     model_id: u32,
//     state: tauri::State<'_, AppState>,
// ) -> CommandResult<(())> {
//     let downloaded_model = state.cached_online_models.lock().await;
//     let model = downloaded_model
//         .iter()
//         .find(|item| item.id == model_id)
//         .ok_or(anyhow!("Could not find model in list"))?;

//     let _ = ModelDownloader::download_model(state.settings.lock().await.app_dir.to_owned(), model)
//         .await;

//     Ok(())
// }

#[derive(Deserialize, specta::Type, Serialize, Debug, Clone)]
pub struct Model {
    pub r#type: String,
    pub name: String,
    pub url: String,
    pub id: u32,
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
    fn load_downloaded_models(model_path: &PathBuf) -> CommandResult<Vec<Model>> {
        let mut models: Vec<Model> = Vec::new();
        let dir = fs::read_dir(model_path.join("tts")).ok();

        // IN PROGRESS: use metadata TOML file

        if let Some(dir) = dir {
            for item in dir {
                let file_name = item.unwrap().file_name();

                models.push(Model {
                    r#type: String::from("vits"),
                    name: file_name.to_string_lossy().to_string(),
                    url: todo!(),
                    id: todo!(),
                });
            }
        }
        Ok(models)
    }
    fn load_cached_model_list(
        cache: Mutex<Vec<Model>>,
        model_path: &PathBuf,
    ) -> CommandResult<Vec<Model>> {
        let mut cache_list = model_path.clone();
        cache_list.push("metadata.toml");
        //TODO elsewhere: when loading cached model list fails, try to refetch the list...
        let downloaded_file =
            fs::read(cache_list).map_err(|err| anyhow!("could not read cached model list"))?;

        let models: Vec<Model> = toml::from_slice(&downloaded_file).map_err(|_| anyhow!(""))?;

        Ok(models)
    }
    async fn get_models(cache: Mutex<Vec<Model>>) -> CommandResult<Vec<Model>> {
        let models = cache.lock().await;

        if models.is_empty() {
            let req: GithubModelResponse = reqwest::get(SHERPA_DOWNLOAD_URL)
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
        } else {
            Ok(models.clone())
        }
    }
    async fn download_model(data_path: PathBuf, model: &Model) -> CommandResult<()> {
        let req = reqwest::blocking::get(model.url.as_str())
            .map_err(|_| anyhow!("Could not download model"))?;
        let archive = req.bytes().map_err(|_| anyhow!("Could not download model"))?;
        let tar = BzDecoder::new(archive.as_bytes());

        let mut decoded = Archive::new(tar);
        decoded
            .unpack(data_path)
            .map_err(|_| anyhow!("Could not unpack downloaded archive."))?;

        Ok(())
    }
}
