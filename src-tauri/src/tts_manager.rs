use crate::{
    config::TtsJobConfig,
    model_manager::{Model, ModelManager},
    recents::RecentDoc,
    text_extraction::get_page_contents,
    tts::TtsGenerationProgress,
    util::CommandResult,
};
use anyhow::{anyhow, Context};
use bzip2::bufread::BzDecoder;
use futures::StreamExt;
use image::EncodableLayout;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sherpa_onnx::{GenerationConfig, OfflineTts};
use tauri::ipc::Channel;
use uuid::Uuid;

use std::{
    default, fs, io::{Read, Write}, path::PathBuf, sync::Arc, time::{Duration, Instant},
};
use tar::Archive;
use tokio::{sync::{Mutex, mpsc}, task::JoinHandle};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{info, instrument};
#[derive(Default, Debug, specta::Type, Clone, Deserialize, Serialize)]
pub enum TTSRequestProgress {
    #[default]
    Queued,
    InProgress(u32),
    Completed
}
#[derive(Debug, specta::Type, Clone, Deserialize, Serialize)]
pub struct TTSRequest {
    pub url: String,
    pub name: String,
    pub progress: TTSRequestProgress
}

#[instrument(skip(progress_reader))]
pub async fn tts_manager(
    data_path: &PathBuf,
    downloaded_models: Arc<Mutex<Vec<Model>>>,
    recent_tts: Arc<Mutex<Vec<RecentDoc>>>,
    mut receiver: mpsc::Receiver<TtsJobConfig>,
    progress_reader: Channel<TtsGenerationProgress>,
) -> CommandResult<()> {
    info!("Starting TTS manager thread");
    // if fs::read_dir("out").is_err() {
    //     fs::create_dir("out")
    //         .context("Failed to create output directory")?;
    // }
    while let Some(job) = receiver.recv().await {
        let downloaded_models = downloaded_models.lock().await.clone();
        let data_path = data_path.clone();
        let progress_reader = progress_reader.clone();
        let recent_tts = recent_tts.clone();
        let _: JoinHandle<CommandResult<()>> = tokio::task::spawn(async move {
            info!("Creating job with config: {:?}", job);
            let mut p = get_page_contents(&job.input_file, job.force_ocr, job.start_page)?;
            p.sort_by_key(|item| item.index);

            let model = ModelManager::get_model(job.voice_id, &downloaded_models).await?;
            let current_path = data_path.clone().join(format!("tts/{}", model.id));
            println!("{:?}", current_path);
            // let dir = fs::read_dir(&current_path).map_err(|_| anyhow!("Could not get dir"))?;
            let config = model.get_config(&data_path)?;
            let tts = OfflineTts::create(&config.try_into()?)
                .ok_or(anyhow!("Failed to create offline TTS"))?;
            let id: String = Uuid::new_v4().into();
            let gen_config = GenerationConfig {
                sid: job.speaker_id as i32,
                speed: job.speed,
                ..Default::default()
            };
            let mut all_text = String::new();
            p.iter().for_each(|item| {
                all_text += &item.contents;
            });
            let reader = progress_reader.clone();
            // debounce so we don't send too many progress updates
            let mut timer = Instant::now();
            let audio = tts
                .generate_with_config(
                    all_text.as_str(),
                    &gen_config,
                    Some(move |samples: &[f32], progress: f32| -> bool {
                        if timer.elapsed() > Duration::from_secs(1) {
                            let _ =
                                progress_reader.send(TtsGenerationProgress::InProgress(progress, Some(samples.to_vec())));
                            timer = Instant::now();
                        }
                        true
                    }),
                )
                .context("TTS Generation failed")?;

            let _ = reader
                .send(TtsGenerationProgress::Finished)
                .map_err(|_| anyhow!("TTS Generation Finished"));
            let file_path = format!(
                "{}/{}",
                job.output_dir
                    .to_str()
                    .ok_or(anyhow!("Failed to get output dir."))?,
                id.clone()
            );
            info!("Saving to: {:?}", file_path);
            let saved = audio.save(file_path.as_str());
            info!("Job Done: status {saved}");
            if saved {
                {
                    let mut data = recent_tts.blocking_lock();
                    data.push(RecentDoc {
                        name: id.clone(),
                        path: file_path.clone(),
                        id,
                    });
                }
            }
                    Ok(())
        });
    }
Ok(())
}
