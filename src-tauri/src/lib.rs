#![feature(path_absolute_method)]
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use anyhow::{anyhow, Context};
use glam::UVec2;
use image::{ColorType, DynamicImage};
use once_cell::sync::Lazy;
use pdf2image::{RenderOptionsBuilder, PDF};
use pdf_inspector::process_pdf;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsVitsModelConfig};
use std::{
    env::current_dir, fmt::{Display, Error, Formatter}, fs::{self}, path::{Path, PathBuf}, sync::Arc, thread::current,
};

use tauri::AppHandle;
use tokio::sync::Mutex;

struct AppState {
    settings: Mutex<TtsAppConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TtsJobConfig {
    /// PDF file to open
    input_file: String,
    /// output file. if multiple, will prefix each file.
    output_prefix: String,
    /// start the narration at a certain page
    start_page: usize,
    /// sets a voice for the narration. see https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/index.html
    // #[arg(long, default_value = "vits-piper-en_US-libritts_r-medium")]
    voice: String,
    /// sets the speed for the speaker
    // #[arg(long, default_value_t = 1.0)]
    speed: f32,

    // #[arg(long, default_value_t = true)]
    combine_pages: bool,

    /// for voices that have multiple speakers, pass a speaker_id.
    // #[arg(short, long, default_value_t = 1)]
    speaker_id: i32,
    use_ocr: bool, // end TTS config here
}
#[derive(Serialize, Deserialize, Clone)]
struct TtsAppConfig {
    /// start the narration at a certain page
    start_page: usize,
    output_prefix: String,
    /// sets a voice for the narration. see https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/index.html
    // #[arg(long, default_value = "vits-piper-en_US-libritts_r-medium")]
    voice: String,
    /// sets the speed for the speaker
    // #[arg(long, default_value_t = 1.0)]
    speed: f32,

    // #[arg(long, default_value_t = true)]
    combine_pages: bool,

    /// for voices that have multiple speakers, pass a speaker_id.
    // #[arg(short, long, default_value_t = 1)]
    speaker_id: i32,
    // end TTS config here
}
impl TtsAppConfig {
    fn load_or_default() -> Self {
        let path = Path::new("config.toml");
        if fs::exists(path).is_ok_and(|item| item) {
            let config = fs::read_to_string("config.toml").unwrap();
            let config: TtsAppConfig = toml::from_str(config.as_str()).unwrap();

            config
        } else {
            let config = TtsAppConfig {
                start_page: 0,
                voice: String::from("vits-piper-en_US-libritts_r-medium"),
                speed: 1.0,
                combine_pages: true,
                speaker_id: 1,
                output_prefix: String::from("page_"),
            };
            fs::write(path, toml::to_string(&config).unwrap()).unwrap();

            config
        }
    }
}

#[derive(Debug)]
struct Page {
    index: u32,
    contents: String,
}
impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Page {}: {}", self.index, self.contents)
    }
}

fn ocr_region(dims: UVec2, coords: UVec2, image: &DynamicImage) -> String {
    let image_dims = UVec2::from_array([image.width(), image.height()]);
    //assert!(image.color() == ColorType::L8);
    let stride_pixel: u32 = match image.color() {
        ColorType::L8 => 1,
        ColorType::La8 => 1,
        ColorType::Rgb8 => 3,
        ColorType::Rgba8 => 4,

        _ => panic!("invalid value"),
    };
    let stride_line: u32 = image_dims.x * stride_pixel;
    let byte_offset: usize = (coords.y * stride_line + coords.x * stride_pixel) as usize;
    assert!(UVec2::cmple(coords + dims, image_dims).all());
    let text = tesseract::ocr_from_frame(
        &image.as_bytes()[byte_offset..],
        dims.x as i32,
        dims.y as i32,
        stride_pixel as i32,
        stride_line as i32,
        "eng",
    )
    .unwrap();
    text
}

fn text_from_image(image: &DynamicImage) -> (String, String) {
    let image_dims = UVec2::from_array([image.width(), image.height()]);
    let old_dims = UVec2::from_array([3099, 2379]);
    let dims = UVec2::from_array([1166, 1809]) * image_dims / old_dims; //TODO
    let left_coords = UVec2::from_array([374, 193]) * image_dims / old_dims; //TODO
    let right_coords = UVec2::from_array([1808, 196]) * image_dims / old_dims; //TODO

    let left_text = post_process_text(&ocr_region(dims, left_coords, image));
    let right_text = post_process_text(&ocr_region(dims, right_coords, image));
    (left_text, right_text)
}
fn post_process_text(string: &str) -> String {
    string.replace("-\n", "").replace("\n", " ")
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    JobFailed(#[from] anyhow::Error),
}
impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn run_job(job: TtsJobConfig) -> Result<(), CommandError> {
    println!("Job Config: {:?}", job);
    let mut p: Vec<Page> = match job.use_ocr {
        true => {
            let mut p = Vec::new();
            let input_path = Path::new(&job.input_file);
            let pdf = process_pdf(input_path);

            if let Some(text) = &pdf
                .map_err(|_| anyhow!("Could not extract text from PDF."))?
                .markdown
            {
                p.push(Page {
                    contents: text.clone(),
                    index: 0,
                })
            };
            p
        }
        false => {
            let pdf = PDF::from_file(&job.input_file).context("Could not read PDF file")?;
            let page_images: Vec<DynamicImage> = pdf
                .render(
                    pdf2image::Pages::Range(job.start_page as u32..=(pdf.page_count() - 1)),
                    RenderOptionsBuilder::default()
                        .greyscale(true)
                        .build()
                        .context("Could not create render options.")?,
                )
                .context("Could not render a pdf into an image.")?
                .iter_mut()
                .map(|page| page.grayscale().rotate90())
                .collect();

            page_images
                .par_iter()
                .enumerate()
                .flat_map(|(file_index, page)| {
                    let (left_text, right_text) = text_from_image(page);
                    [
                        Page {
                            index: 2 * file_index as u32,
                            contents: left_text,
                        },
                        Page {
                            index: 2 * file_index as u32 + 1,
                            contents: right_text,
                        },
                    ]
                })
                .collect()
        }
    };
    p.sort_by_key(|item| item.index);

    if fs::read_dir("out").is_err() {
        fs::create_dir("out").context("Failed to create output directory")?;
    }
    println!("{:?}", p);
    let binding = std::env::current_dir().context("Error finding TTS Model path")?;
    let current_path = binding
        .parent()
        .ok_or(anyhow!("Error finding TTS Model path"))?;
    // let current_path = Path::new();
    let mut dir = fs::read_dir(current_path.join(&job.voice)).context("No TTS Model")?;

    if dir.next().is_none() {
        return Err(anyhow!("ASAAS").into());
    }
    // TODO: FIX THIS TO USE ACTUAL GOOD PATHS.
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: Some(format!("./tts/{}/en_US-libritts_r-medium.onnx", job.voice)),
                tokens: Some(format!("./tts/{}/en_US-libritts_r-medium.onnx", job.voice)),
                data_dir: Some(format!("./tts/{}/espeak-ng-data", job.voice)),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: 1,
            debug: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let text = "Hello World!";
    let tts = OfflineTts::create(&config).expect("Failed to create OfflineTts");
    let gen_config = GenerationConfig {
        sid: 1,
        speed: 1.0,
        ..Default::default()
    };
    println!("Somehow we're past here");
    let audio = tts
        .generate_with_config(
            text,
            &gen_config,
            Some(|_samples: &[f32], progress: f32| -> bool {
                println!("Progress: {:.1}%", progress * 100.0);
                true
            }),
        )
        .expect("Generation failed");

    let saved = audio.save("file.wav");
    println!("Saved: {saved}");

    Ok(())
}
#[tauri::command]
async fn get_config(state: tauri::State<'_, AppState>) -> TtsAppConfig {
    // TODO: figure out best practice for returning data based on mutex
    let config: TtsAppConfig = {
    let config = state.settings.lock().await;
        config.clone()
    };

    config

}
fn setup(app: &AppHandle) {
    // app.
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(TtsAppConfig::load_or_default()),
        })
        .setup(move |app| {
            setup(app.handle());
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![get_config, run_job])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
