// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use anyhow::anyhow;
use glam::UVec2;
use image::{ColorType, DynamicImage, EncodableLayout, GrayImage, imageops};
use pdf2image::{PDF, RenderOptionsBuilder};
use rayon::prelude::*;
use sherpa_onnx::{OfflineTts, GenerationConfig, OfflineTtsVitsModelConfig, OfflineTtsConfig, OfflineTtsModelConfig};
use std::{
    fmt::{Display, Error, Formatter},
    fs::{self},
};

use regex;

pub fn extract_number(file_path: &str) -> Option<u32> {
    let re = regex::Regex::new(r"\d+").unwrap();
    if let Some(captures) = re.captures(file_path) {
        captures.get(0).and_then(|m| m.as_str().parse::<u32>().ok())
    } else {
        None
    }
}

struct TtsJobConfig {
    /// PDF file to open
    input_file: String,
    /// output file. if multiple, will prefix each file.
    output_file: String,
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

    // end TTS config here


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
        //ColorType::L16=>2,
        //ColorType::La16=>4,
        //ColorType::Rgb16=>48,
        //ColorType::Rgba16=>64,
        //ColorType::Rgb32F=>96,
        //ColorType::Rgba32F=>128,
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

    let mut left_text = post_process_text(&ocr_region(dims, left_coords, &image));
    let mut right_text = post_process_text(&ocr_region(dims, right_coords, &image));
    (left_text, right_text)
}
fn post_process_text(string: &String) -> String {
    string.replace("-\n", "").replace("\n", " ")
}

fn run_job(job: TtsJobConfig) -> Result<(), anyhow::Error> {
    let pdf = PDF::from_file(&job.input_file).unwrap();
    let page_images: Vec<DynamicImage> = pdf
        .render(
            pdf2image::Pages::Range(job.start_page as u32..=(pdf.page_count() - 1)),
            RenderOptionsBuilder::default().greyscale(true).build()?,
        )?
        .iter_mut()
        .map(|page| page.grayscale().rotate90())
        .collect();

    let mut pages: Vec<Page> = page_images
        .par_iter()
        .enumerate()
        .flat_map(|(file_index, page)| {
            let (left_text, right_text) = text_from_image(page);
            [
                Page {
                    index: 2 * file_index as u32 + 0,
                    contents: left_text,
                },
                Page {
                    index: 2 * file_index as u32 + 1,
                    contents: right_text,
                },
            ]
        })
        .collect();
    pages.sort_by_key(|item| item.index);

    if fs::read_dir("out").is_err() {
        fs::create_dir("out").unwrap();
    }

    let mut dir = fs::read_dir(format!("tts/{}", job.voice)).expect("No TTS Model!");

    if dir.next().is_none() {
        return Err(anyhow!("Couldn't find tts model"));
    }
    let config = OfflineTtsConfig {
        model: sherpa_onnx::OfflineTtsModelConfig {
            vits: OfflineTtsVitsModelConfig {
                model: Some(
                    "./tts/vits-piper-en_US-libritts_r-medium/en_US-libritts_r-medium.onnx".to_string()
                ),
                tokens: Some(
                    "./tts/vits-piper-en_US-libritts_r-medium/en_US-libritts_r-medium.onnx".to_string(),
                ),
                data_dir: Some("./tts/vits-piper-en_US-libritts_r-medium/espeak-ng-data".to_string()),
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
    let audio = tts
        .generate_with_config(
            &text,
            &gen_config,
            Some(|_samples: &[f32], progress: f32| -> bool {
                println!("Progress: {:.1}%", progress * 100.0);
                true
            }),
        )
        .expect("Generation failed");

     // new code:
    //      if audio.save(&args.output) {
    //     println!("Saved to: {}", args.output);
    // } else {
    //     eprintln!("Failed to save {}", args.output);
    // }
    // let sample_rate = complete_pages[0].sample_rate;
    // match args.combine_pages {
    //     true => do something
    //     }
    // }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
