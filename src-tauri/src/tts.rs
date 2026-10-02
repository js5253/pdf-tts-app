use std::{fs, time::Duration};

use crate::{
    config::{self},
    recents::RecentDoc,
    text_extraction::get_page_contents,
    AppState, CommandResult, TtsGenerationProgress,
};
use anyhow::{anyhow, Context};
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsVitsModelConfig};
use tokio::time::Instant;
use uuid::Uuid;

#[tauri::command(async)]
#[specta::specta]
pub async fn run_job(
    job: config::TtsJobConfig,
    state: tauri::State<'_, AppState>,
    progress_reader: tauri::ipc::Channel<TtsGenerationProgress>,
) -> CommandResult<()> {
    println!("Job Config: {:?}", job);
    let mut p = get_page_contents(&job.input_file, job.use_ocr, job.start_page)?;
    p.sort_by_key(|item| item.index);

    println!("{:?}", p);
    if fs::read_dir("out").is_err() {
        fs::create_dir("out").context("Failed to create output directory")?;
    }
    let binding = state.settings.lock().await;
    let current_path = binding
        .app_dir
        .parent()
        .ok_or(anyhow!("Error finding TTS Model path"))?;
    let current_path = &current_path.join("tts/");
    let current_path = &current_path.join(&job.voice);
    println!("{:?}", current_path);
    // let current_path = Path::new();
    let mut dir = fs::read_dir(current_path).context("Could not read the TTS model directory")?;
    if dir.next().is_none() {
        return Err(anyhow!("Could not find a TTS model").into());
    }
    let mut model_path: std::path::PathBuf = current_path.clone();
    model_path.push("en_US-libritts_r-medium.onnx");

    let mut token_path: std::path::PathBuf = current_path.clone();
    token_path.push("tokens.txt");

    let mut data_dir = current_path.clone();
    data_dir.push("espeak-ng-data");
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
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
                ..Default::default()
            },
            num_threads: std::thread::available_parallelism()
                .map_err(|_| anyhow!("could not get thread count for tts generation"))?
                .get() as i32,
            debug: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let tts = OfflineTts::create(&config).ok_or(anyhow!("Could not create TTS Engine"))?;
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
                    let _ = progress_reader.send(TtsGenerationProgress::InProgress(progress));
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
    let saved = audio.save(file_path.as_str());
    println!("Done!!!!");
    match saved {
        true => {
            state.recent_tts.blocking_lock().push(RecentDoc {
                name: id.clone(),
                path: file_path,
                id,
            });
            Ok(())
        }
        false => Err(anyhow!("failed to save audio file").into()),
    }
}
