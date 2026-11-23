use crate::model_manager::ModelManager;
use crate::models::DownloadManagerProgress;
use crate::{model_manager::Model, util::CommandResult};
use anyhow::anyhow;
use futures::StreamExt;
use image::EncodableLayout;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::ipc::Channel;
use tokio::{fs::File, io::AsyncWriteExt};
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownloadRequestType {
    Download,
    CancelDownload,
}
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub id: u32,
    pub r#type: DownloadRequestType,
}
#[instrument(skip(progress))]
pub async fn download_manager(
    data_path: &PathBuf,
    downloaded_models: Arc<Mutex<HashMap<u32, Model>>>,
    mut receiver: mpsc::Receiver<DownloadRequest>,
    progress: Channel<DownloadManagerProgress>,
) -> CommandResult<()> {
    info!("Starting download manager thread");
    let cancellation_tokens: Arc<Mutex<HashMap<u32, CancellationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let thread_token = Arc::clone(&cancellation_tokens);
    while let Some(req) = receiver.recv().await {
        let data_path = data_path.clone();
        let thread_token = Arc::clone(&thread_token);
        let downloaded_models = Arc::clone(&downloaded_models);
        match req.r#type {
            DownloadRequestType::CancelDownload => {
                let token = cancellation_tokens.lock().await;
                let token = token.get(&req.id).ok_or(anyhow!(
                    "Failed to get cancellation token for model download"
                ))?;
                token.cancel();
            }
            DownloadRequestType::Download => {
                let a: Option<Model> = {
                    let binding = downloaded_models.lock().await;
                    let binding = binding.get(&req.id);
                    binding.map(|item| item.to_owned())
                };
                let Some(model) = a else {
                    return Ok(());
                };
                let progress = progress.clone();
                let _: JoinHandle<CommandResult<()>> = tokio::task::spawn(async move {
                    let token = CancellationToken::new();
                    info!("Request for: Name {:?} url {:?}", &model.name, &model.url);
                    let client = reqwest::Client::new();
                    let mut req_path = data_path.clone();
                    req_path.push("tts/");
                    req_path.push(&model.name);
                    let mut dest = File::create(&req_path).await?;

                    let res = client
                        .get(&model.url)
                        .header("User-Agent", "js5253-pdf-tts-app")
                        .bearer_auth(dotenvy_macro::dotenv!("GH_ACCESS_TOKEN"))
                        .send()
                        .await
                        //         .map_err(|err| {
                        //             let _unused = anyhow!("Could not download model");
                        //             error!("{:?}", err);
                        // })
                        .expect("msg");
                    let mut stream = res.bytes_stream();
                    let mut offset: usize = 0;
                    let progress = progress.clone();
                    while let Some(bytes) = stream.next().await {
                        if token.is_cancelled() {
                            return Ok(());
                        }
                        // transmit download progress here
                        info!("download - current offset {}", offset);
                        let chunk = bytes?;
                        let _ = dest
                            .write_all(chunk.as_bytes())
                            .await
                            .or(Err("Error writing to file"));
                        let prev = offset;
                        offset += chunk.len();
                        if offset > prev - 5000 {
                            let _unused = progress.send(DownloadManagerProgress {
                                name: model.name.clone(),
                                id: model.id,
                                progress: offset / model.size,
                                r#type: String::from("inProgress"),
                            });
                        }
                    }
                    let _unused = progress.send(DownloadManagerProgress {
                        name: model.name.clone(),
                        id: model.id,
                        progress: offset / model.size,
                        r#type: String::from("done"),
                    });
                    info!("Downloaded file {:?} at {:?}", &req_path.to_str(), dest);
                    let _ = thread_token.lock().await.insert(model.id, token);
                    {
                        let mut models = downloaded_models.lock().await;
                        models.insert(model.id, model);
                        let _ = ModelManager::set_model_downloaded(models.deref_mut(), &data_path);
                    };
                    Ok(())
                });
            }
        }
    }
    Ok(())
}
