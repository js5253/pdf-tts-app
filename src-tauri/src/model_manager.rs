use crate::{
    download_manager::DownloadRequest,
    util::{CommandError, CommandResult, Error},
    AppState,
};
use anyhow::{anyhow, Context};
use futures::TryStreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sherpa_onnx::{
    OfflineTtsConfig, OfflineTtsKittenModelConfig, OfflineTtsKokoroModelConfig,
    OfflineTtsMatchaModelConfig, OfflineTtsModelConfig, OfflineTtsSupertonicModelConfig,
    OfflineTtsVitsModelConfig,
};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{info, instrument};

const SHERPA_URL: &str = "https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/tts-models";

pub async fn load_cached_models(data_path: &mut PathBuf) -> CommandResult<HashMap<u32, Model>> {
    data_path.push("models.toml");
    let contents = fs::read_to_string(data_path)?;
    let contents = contents.as_str();
    let models: HashMap<u32, Model> = toml::from_str(contents)?;
    Ok(models)
}

#[tauri::command(async)]
#[specta::specta]
#[instrument(skip(state))]
pub async fn get_online_models(state: tauri::State<'_, AppState>) -> CommandResult<Vec<Model>> {
    ModelManager::get_models(
        state.gh_access_token,
        &state.app_dir,
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
    let _ = ModelManager::download_model(model, &state.download_manager).await;

    Ok(())
}

#[derive(Deserialize, specta::Type, Serialize, Debug, Clone)]
pub struct Model {
    pub r#type: SupportedTTSType,
    pub name: String,
    pub url: String,
    pub id: u32,
}
impl Model {
    pub fn get_config(&self, base_dir: &Path) -> CommandResult<TTSModelConfig> {
        let base_dir = base_dir.to_path_buf();

        let mut token_path = base_dir.clone();
        token_path.push("tokens.txt");

        let mut data_dir = base_dir.clone();
        data_dir.push("espeak-ng-data");

        match &self.r#type {
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
                            .map_err(|_| anyhow!("Failed to get token path"))?,
                    ),
                    data_dir: Some(
                        data_dir
                            .into_string()
                            .map_err(|_| anyhow!("Failed to get data dir"))?,
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

impl TryInto<OfflineTtsConfig> for TTSModelConfig {
    type Error = crate::util::CommandError;

    fn try_into(self) -> Result<OfflineTtsConfig, Self::Error> {
        let num_threads = std::thread::available_parallelism()
            .map_err(|_| anyhow!("could not get thread count for tts generation"))?
            .get() as i32;
        Ok(OfflineTtsConfig {
            model: match self.r#type {
                SupportedTTSType::Vits => OfflineTtsModelConfig {
                    vits: OfflineTtsVitsModelConfig {
                        model: self.model,
                        lexicon: self.lexicon,
                        tokens: self.tokens,
                        data_dir: self.data_dir,
                        noise_scale: self.noise_scale,
                        noise_scale_w: self.noise_scale_w,
                        length_scale: self.length_scale,
                        dict_dir: self.dict_dir,
                    },

                    debug: true,
                    ..Default::default()
                },
                SupportedTTSType::Kitten => OfflineTtsModelConfig {
                    kitten: OfflineTtsKittenModelConfig {
                        model: self.model,
                        tokens: self.tokens,
                        voices: self.dict_dir, // recheck this
                        data_dir: self.data_dir,
                        length_scale: self.length_scale,
                    },
                    ..Default::default()
                },
                SupportedTTSType::Matcha => OfflineTtsModelConfig {
                    matcha: OfflineTtsMatchaModelConfig {
                        acoustic_model: self.model,
                        tokens: self.tokens,
                        data_dir: self.data_dir,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                SupportedTTSType::Kokoro => OfflineTtsModelConfig {
                    kokoro: OfflineTtsKokoroModelConfig {
                        model: self.model,
                        tokens: self.tokens,
                        data_dir: self.data_dir,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                SupportedTTSType::Supertonic => OfflineTtsModelConfig {
                    supertonic: OfflineTtsSupertonicModelConfig {
                        // model: self.model,
                        ..Default::default() // vocoder: self.,
                                             // lexicon: todo!(),
                                             // tokens: todo!(),
                                             // data_dir: todo!(),
                                             // noise_scale: todo!(),
                                             // length_scale: todo!(),
                                             // dict_dir: todo!(),
                    },
                    num_threads,
                    ..Default::default()
                },
            },
            ..Default::default()
        })
    }
}

pub struct ModelManager;
impl ModelManager {
    #[instrument]
    pub async fn get_model(
        id: u32,
        cached_online_models: &Arc<Mutex<HashMap<u32, Model>>>,
    ) -> CommandResult<Model> {
        let cached_online_models = cached_online_models.lock().await;
        // let model = cached_online_models.get(&id).ok_or(Err(Error("Failed to get TTS model".to_owned())))?
        todo!()
    }
    #[instrument]
    async fn get_models(
        token: &str,
        app_dir: &PathBuf,
        force_recheck: bool,
        cached_online_models: Arc<Mutex<HashMap<u32, Model>>>,
    ) -> CommandResult<Vec<Model>> {
        let mut cached_online_models = cached_online_models.lock().await;
        if !cached_online_models.is_empty() && !force_recheck {
            info!("Using cached online models");
            let arr: Vec<Model> = cached_online_models.clone().into_values().collect();
            Ok(arr)
        } else {
            info!("getting online models... {}", token);
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
                .error_for_status()
                .map_err(|e| {
                    info!("Failed to get HTTP request: {}", e);
                    anyhow!("Could not download model list.")
                })?
                .json::<GithubModelResponse>()
                .await
                .context("Failed to deserialize GitHub model response.")?;

            // serialize into
            let mut output_path = app_dir.clone();
            output_path.push("cache.toml");

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
            fs::write(output_path, toml::to_string_pretty(&models)?)?;

            info!("Models: {:?}", models);
            *cached_online_models = models.clone();
            let arr: Vec<Model> = models.into_values().collect();

            Ok(arr)
        }
    }
    async fn download_model(
        model: &Model,
        tx: &mpsc::Sender<DownloadRequest>,
    ) -> CommandResult<()> {
        tx.send(DownloadRequest {
            url: model.url.clone(),
        })
        .await?;
        Ok(())
    }
}
