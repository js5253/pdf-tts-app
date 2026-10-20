use std::{fs, time::Duration};

use crate::model_manager::SupportedTTSType;
use crate::{
    config::{self},
    model_manager::get_model_config,
    recents::RecentDoc,
    text_extraction::get_page_contents,
};
use crate::{util::CommandResult, AppState, TtsGenerationProgress};
use anyhow::{anyhow, Context};
use log::info;
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig};
use tokio::time::Instant;
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(state, progress_reader), err)]
#[tauri::command(async)]
#[specta::specta]
pub async fn run_job(
    job: config::TtsJobConfig,
    state: tauri::State<'_, AppState>,
    progress_reader: tauri::ipc::Channel<TtsGenerationProgress>,
) -> CommandResult<()> {
    info!("Creating job with config: {:?}", job);
    let mut p = get_page_contents(&job.input_file, job.use_ocr, job.start_page)?;
    p.sort_by_key(|item| item.index);

    if fs::read_dir("out").is_err() {
        fs::create_dir("out").context("Failed to create output directory")?;
    }

    let current_path = state.app_dir.clone().join("tts/").join(&job.voice);
    println!("{:?}", current_path);
    // let current_path = Path::new();
    let mut dir = fs::read_dir(&current_path).context(format!(
        "Could not read the TTS model directory: {:?}",
        current_path
    ))?;
    if dir.next().is_none() {
        return Err(anyhow!("Could not find a TTS model").into());
    }
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            vits: get_model_config(&current_path, SupportedTTSType::Vits)?
                .try_into()
                .map_err(|_| anyhow!("Failed to get model config"))?,
            num_threads: std::thread::available_parallelism()
                .map_err(|_| anyhow!("could not get thread count for tts generation"))?
                .get() as i32,
            debug: true,
            ..Default::default()
        },
        ..Default::default()
    };
    dbg!("{}", &config);
    let tts = OfflineTts::create(&config).ok_or(anyhow!(
        "Could not create TTS Engine with config {:?}",
        config
    ))?;
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
    info!("Saving to: {:?}", file_path);
    let saved = audio.save(file_path.as_str());
    info!("Job Done: status {saved}");
    // if saved {
    //     {
    //         {
    //             let mut data = state.recent_tts.lock().await;
    //             data.push(RecentDoc {
    //                 name: id.clone(),
    //                 path: file_path.clone(),
    //                 id: id.clone(),
    //             });
    //         }
    //     }
    // }
    Ok(())
}
