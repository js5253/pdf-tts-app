// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use anyhow::{anyhow, Context};
use glam::UVec2;
use image::{ColorType, DynamicImage};
use markdown_strip::strip_markdown;
use pdf2image::{RenderOptionsBuilder, PDF};
use pdf_inspector::process_pdf;
use rayon::prelude::*;
use serde::{Serialize};
use tauri::Manager;
use sherpa_onnx::{GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsVitsModelConfig};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use std::{
    fmt::{self, Display, Formatter},
    fs::{self},
    path::Path,
    time::Duration,
    env
};
mod config;
use tauri_specta::{collect_commands, Builder};

use tokio::{sync::Mutex, time::Instant};

use crate::config::TtsAppConfig;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, specta::Type, Serialize)]
pub struct Error(String);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Self(format!("{error:#}"))
    }
}

struct AppState {
    settings: Mutex<TtsAppConfig>,
}

#[derive(Debug)]
struct Page {
    index: u32,
    contents: String,
}
impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Page {}: {}", self.index, self.contents)
    }
}

fn ocr_region(dims: UVec2, coords: UVec2, image: &DynamicImage) -> Result<String> {
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
    .context("Failed to extract text from a PDF image")?;
    Ok(text)
}

fn text_from_image(image: &DynamicImage) -> (String, String) {
    let image_dims = UVec2::from_array([image.width(), image.height()]);
    let old_dims = UVec2::from_array([3099, 2379]);
    let dims = UVec2::from_array([1166, 1809]) * image_dims / old_dims; //TODO
    let left_coords = UVec2::from_array([374, 193]) * image_dims / old_dims; //TODO
    let right_coords = UVec2::from_array([1808, 196]) * image_dims / old_dims; //TODO

    let left_text = post_process_text(&ocr_region(dims, left_coords, image).unwrap());
    let right_text = post_process_text(&ocr_region(dims, right_coords, image).unwrap());
    (left_text, right_text)
}
fn post_process_text(string: &str) -> String {
    string.replace("-\n", "").replace("\n", " ")
}
#[derive(Clone, Serialize, specta::Type)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
enum TtsGenerationProgress {
    InProgress(f32),
    Finished,
}

fn run_pdf_text(input_file: &String) -> Result<Vec<Page>> {
    let mut p: Vec<Page> = Vec::new();
    let input_path = Path::new(&input_file);
    let pdf = process_pdf(input_path);

    if let Some(text) = &pdf
        .map_err(|_| anyhow!("Could not extract text from PDF."))?
        .markdown
    {
        p.push(Page {
            contents: strip_markdown(&text.clone()),
            index: 0,
        })
    };
    Ok(p)
}
fn run_pdf_ocr(input_file: &String, start_page: u32) -> Result<Vec<Page>> {
    let pdf = PDF::from_file(input_file).context("Could not read PDF file")?;
    let page_images: Vec<DynamicImage> = pdf
        .render(
            pdf2image::Pages::Range(start_page..=(pdf.page_count() - 1)),
            RenderOptionsBuilder::default()
                .greyscale(true)
                .build()
                .context("Could not create render options.")?,
        )
        .context("Could not render a pdf into an image.")?
        .iter_mut()
        .map(|page| page.grayscale().rotate90())
        .collect();

    let pages: Vec<Page> = page_images
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
        .collect();
    Ok(pages)
}
fn get_page_contents(input_file: &String, use_ocr: bool, start_page: u32) -> Result<Vec<Page>> {
    match &input_file.contains(".pdf") {
        true => match use_ocr {
            true => run_pdf_ocr(input_file, start_page),
            false => run_pdf_text(input_file),
        },
        false => {
            let contents = anydoc::to_markdown(input_file)
                .map_err(|_| anyhow!("could not get markdown from PDF"))?;
            let contents = strip_markdown(&contents.to_string());
            Ok(vec![Page { index: 0, contents }])
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
async fn run_job(
    job: config::TtsJobConfig,
    progress_reader: tauri::ipc::Channel<TtsGenerationProgress>,
) -> Result<()> {
    println!("Job Config: {:?}", job);
    let mut p = get_page_contents(&job.input_file, job.use_ocr, job.start_page)?;
    p.sort_by_key(|item| item.index);

    println!("{:?}", p);
    if fs::read_dir("out").is_err() {
        fs::create_dir("out").context("Failed to create output directory")?;
    }
    let binding = std::env::current_dir().context("Error finding TTS Model path")?;
    let current_path = binding
        .parent()
        .ok_or(anyhow!("Error finding TTS Model path"))?;
    let current_path = &current_path.join("tts/");
    let current_path = &current_path.join(&job.voice);
    println!("{:?}", current_path);
    // let current_path = Path::new();
    let mut dir = fs::read_dir(current_path).context("No TTS Model")?;
    if dir.next().is_none() {
        return Err(anyhow!("ASAAS").into());
    }
    let mut model_path: std::path::PathBuf = current_path.clone();
    model_path.push("en_US-libritts_r-medium.onnx");

    let mut token_path: std::path::PathBuf = current_path.clone();
    token_path.push("tokens.txt");

    let mut data_dir = current_path.clone();
    data_dir.push("espeak-ng-data");
    dbg!(token_path.clone(), data_dir.clone());
    // TODO: FIX THIS TO USE ACTUAL GOOD PATHS.
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: Some(model_path.clone().into_os_string().into_string().unwrap()),
                tokens: Some(token_path.into_os_string().into_string().unwrap()),
                data_dir: Some(data_dir.into_os_string().into_string().unwrap()),
                noise_scale: 0.667,
                noise_scale_w: 0.8,
                length_scale: 1.0,
                ..Default::default()
            },
            num_threads: std::thread::available_parallelism().map_err(|_| anyhow!("could not get thread count for tts generation"))?.get() as i32,
            debug: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let tts = OfflineTts::create(&config).ok_or(anyhow!("Could not create TTS Engine"))?;
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
    reader.send(TtsGenerationProgress::Finished).unwrap();

    let saved = audio.save(&format!("{}{}", &job.output_dir, "/output.wav"));
    match saved {
        true => Ok(()),
        false => Err(anyhow!("failed to save audio file").into())
    }
}
#[tauri::command]
#[specta::specta]
async fn get_config(state: tauri::State<'_, AppState>) -> Result<TtsAppConfig> {
    // TODO: figure out best practice for returning data based on mutex
    let config: TtsAppConfig = {
        let config = state.settings.lock().await;
        config.clone()
    };
    Ok(config)
}
#[tauri::command]
#[specta::specta]
fn get_downloaded_models() -> Result<Vec<String>> {
    let mut models: Vec<String> = Vec::new();
    let binding = env::current_dir().context("could not open tts model path")?;
    let model_path = binding
        .parent()
        .ok_or(anyhow!("could not open tts model path"))?;

    let dir = fs::read_dir(model_path.join("tts")).context("No TTS Model")?;
    dir.for_each(|item| models.push(item.unwrap().file_name().to_string_lossy().to_string()));

    Ok(models)
}

#[tauri::command]
#[specta::specta]
async fn set_default_model(model_name: String, state: tauri::State<'_, AppState>) -> Result<()> {
    state.settings.lock().await.voice = model_name;
    Ok(())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![
            set_default_model,
            get_config,
            get_downloaded_models,
            run_job
        ]);
    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .setup(move |app| {
            if let Ok(app_dir) = app.path().app_data_dir() {
                if !app_dir.exists() {
                    let model_path = &app_dir.clone().push("tts");
                    let output_path = &app_dir.clone().push("out");
                    
                    fs::create_dir_all(&output_path)
                    .expect("Failed to create output path.");
                    fs::create_dir_all(&app_dir)
                    .expect("Failed to create App Dir");
                    fs::create_dir_all(&model_path)
                    .expect("Failed to create Model Dir.");
                }
            app.manage(AppState {
                settings: Mutex::new(TtsAppConfig::load_or_default(&app_dir.as_path())),
            });
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_default_model,
            get_downloaded_models,
            run_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
