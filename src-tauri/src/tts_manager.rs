use crate::{config::TtsJobConfig, model_manager::{Model, ModelManager}, text_extraction::get_page_contents, util::CommandResult};
use bzip2::bufread::BzDecoder;
use futures::StreamExt;
use image::EncodableLayout;
use reqwest::Client;
use anyhow::{anyhow, Context};
use sherpa_onnx::{GenerationConfig, OfflineTts};
use uuid::Uuid;

use std::{
    fs, io::{Read, Write}, path::PathBuf, sync::Arc, time::Instant,
};
use tar::Archive;
use tokio::sync::{mpsc, Mutex};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{info, instrument};
#[derive(Debug)]
pub struct TtsRequest {
    pub url: String,
    pub name: String,
}
#[instrument]
pub async fn tts_manager(
    data_path: &PathBuf,
    downloaded_models: Arc<Mutex<Vec<Model>>>,
    mut receiver: mpsc::Receiver<TtsJobConfig>,
) -> CommandResult<()> {
    info!("Starting TTS manager thread");
    if fs::read_dir("out").is_err() {
        fs::create_dir("out").context("Failed to create output directory")?;
    }
    while let Some(job) = receiver.recv().await {
        tokio::task::spawn(async move {
            info!("Creating job with config: {:?}", job);
            let mut p = get_page_contents(&job.input_file, job.use_ocr, job.start_page)?;
            p.sort_by_key(|item| item.index);

            let model = {
                let downloaded_models = downloaded_models.lock().await.clone();
                let m = ModelManager::get_model(job.voice_id, &downloaded_models).await;

                m
            }?;
            let current_path = app_dir.clone().join(format!("tts/{}", model.id));
            println!("{:?}", current_path);
            let mut dir = fs::read_dir(&current_path).context(format!(
                "Could not read the TTS model directory: {:?}",
                current_path
            ))?;
            if dir.next().is_none() {
                return Err(anyhow!("Could not find a TTS model").into());
            }
            let config = model.get_config(&state.app_dir)?;
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
                    Some(move |_samples: &[f32], progress: f32| -> bool {
                        if timer.elapsed() > Duration::from_secs(1) {
                            let _ =
                                progress_reader.send(TtsGenerationProgress::InProgress(progress));
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
                    let mut data = state.recent_tts.blocking_lock();
                    data.push(RecentDoc {
                        name: id.clone(),
                        path: file_path.clone(),
                        id,
                    });
                }
            }
        });
        Ok(())
    }
}
