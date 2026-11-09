use crate::{model_manager::Model, util::CommandResult};
use anyhow::anyhow;
use bzip2::bufread::BzDecoder;
use futures::StreamExt;
use image::EncodableLayout;
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};
use std::{
    io::{Read, Write}, path::PathBuf, sync::Arc};
use tar::Archive;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, instrument};
#[derive(Debug)]
pub struct DownloadRequest {
    pub url: String,
    pub name: String
}
#[instrument]
pub async fn download_manager(
    data_path: &PathBuf,
    downloaded_models: Arc<Mutex<Vec<Model>>>,
    mut receiver: mpsc::Receiver<DownloadRequest>,
) -> CommandResult<()> {
    info!("Starting download manager thread");
    while let Some(req) = receiver.recv().await {
        let data_path = data_path.clone();

        tokio::task::spawn(async move {
            info!("Request for: {:?}", &req);
            let client = reqwest::Client::new();
            let mut req_path = data_path.clone();
            req_path.push("tts/");
            req_path.push(&req.name);
            let Ok(mut dest) = File::create(&req_path).await else {
                info!("Failed to create file at path {:?}", &req_path);
                return;
            };

            let res = client
                .get(req.url)
                .header("User-Agent", "js5253-pdf-tts-app")
                .bearer_auth(dotenvy_macro::dotenv!("GH_ACCESS_TOKEN"))
                .send()
                .await
                .map_err(|_| anyhow!("Could not download model"))
                .expect("msg");
            let mut stream = res.bytes_stream();
            let mut offset = 0;
            while let Some(bytes) = stream.next().await {
                            // transmit download progress here
                info!("download - current offset {}", offset);
                let chunk = bytes.expect("Failed to get chunk");
                let _ = dest
                    .write_all(chunk.as_bytes())
                    .await
                    .or(Err("Error writing to file"));
                offset += chunk.len();
            }
            info!("Downloaded file {:?} at {:?}", &req_path.to_str(), dest);
            // save thing here
        });
    }
    Ok(())
}
