use crate::{model_manager::Model, util::CommandResult};
use anyhow::anyhow;
use futures::StreamExt;
use image::EncodableLayout;
use log::error;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
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
    pub url: Option<String>,
    pub name: Option<String>,
    pub r#type: DownloadRequestType,
}
#[instrument]
pub async fn download_manager(
    data_path: &PathBuf,
    downloaded_models: Arc<Mutex<HashMap<u32, Model>>>,
    mut receiver: mpsc::Receiver<DownloadRequest>,
) -> CommandResult<()> {
    info!("Starting download manager thread");
    let cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let thread_token = Arc::clone(&cancellation_tokens);
    while let Some(req) = receiver.recv().await {
        let data_path = data_path.clone();
        let r = req.clone();
        let thread_token = Arc::clone(&thread_token);

        match req.r#type {
            DownloadRequestType::CancelDownload => {
                let Some(id) = req.name else {
                    return Err(anyhow!("Could not get ID for model download cancellation").into());
                };
                let token = cancellation_tokens.lock().await;
                let token = token.get(&id).ok_or(anyhow!(
                    "Failed to get cancellation token for model download"
                ))?;
                token.cancel();
            }
            DownloadRequestType::Download => {
                let _: JoinHandle<CommandResult<()>> = tokio::task::spawn(async move {
                    let token = CancellationToken::new();
                    let (Some(url), Some(name)) = (req.url, req.name) else {
                        return Err(anyhow!("Failed").into());
                    };
                    info!("Request for: Name {:?} url {:?}", &name, &url);
                    let client = reqwest::Client::new();
                    let mut req_path = data_path.clone();
                    req_path.push("tts/");
                    req_path.push(name);
                    let mut dest = File::create(&req_path).await?;

                    let res = client
                        .get(url)
                        .header("User-Agent", "js5253-pdf-tts-app")
                        .bearer_auth(dotenvy_macro::dotenv!("GH_ACCESS_TOKEN"))
                        .send()
                        .await
                        .map_err(|err| {
                            let _unused = anyhow!("Could not download model");
                            error!("{:?}", err);
                })
                        .expect("msg");
                    let mut stream = res.bytes_stream();
                    let mut offset = 0;
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
                        offset += chunk.len();
                    }
                    info!("Downloaded file {:?} at {:?}", &req_path.to_str(), dest);
                    if let Some(name) = r.name {
                        let _ = thread_token.lock().await.insert(name, token);
                    }
                    Ok(())
                });
            }
        }
    }
    Ok(())
}
