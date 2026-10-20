use std::path::PathBuf;

use anyhow::anyhow;
use bzip2::bufread::BzDecoder;
use image::EncodableLayout;
use tar::Archive;
use tokio::sync::{mpsc};
use tracing::{info, instrument};

use crate::util::CommandResult;
#[derive(Debug)]
pub struct DownloadRequest {
    pub url: String,
}
#[instrument]
pub async fn download_manager(
    data_path: &PathBuf,
    mut receiver: mpsc::Receiver<DownloadRequest>,
) -> CommandResult<()> {
    info!("Starting download manager thread");
    while let Some(req) = receiver.recv().await {
        let data_path = data_path.clone();

        tokio::task::spawn_blocking(move || {
            let client = reqwest::blocking::Client::new();
            let req = client
                .get(req.url)
                .header("User-Agent", "js5253-pdf-tts-app")
                .bearer_auth(dotenvy_macro::dotenv!("GH_ACCESS_TOKEN"))
                .send()
                .map_err(|_| anyhow!("Could not download model"))
                .expect("msg");
            let archive = req.bytes().map_err(|_| anyhow!("A")).unwrap();
            let tar = BzDecoder::new(archive.as_bytes());

            let mut decoded = Archive::new(tar);
            decoded
                .unpack(data_path)
                .map_err(|_| anyhow!("Could not unpack downloaded archive."))
                .unwrap();
        });
    }
    Ok(())
}
