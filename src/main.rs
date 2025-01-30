use anyhow::anyhow;
use glam::UVec2;
use image::{imageops, ColorType, DynamicImage, EncodableLayout, GrayImage};
use pdf2image::{RenderOptionsBuilder, PDF};
use rayon::prelude::*;
use sherpa_rs::tts::{TtsAudio, VitsTts, VitsTtsConfig};
use std::{
    fmt::{Display, Error, Formatter},
    fs::{self},
    ptr::null,
};

use clap::Parser;

/// Program that allows you to use TTS from OCRed PDFs
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDF file to open
    #[arg(short, long)]
    input_file: String,
    /// output file. if multiple, will prefix each file.
    #[arg(short, long, default_value = "idk")]
    output_file: String,
    /// start the narration at a certain page
    #[arg(long, default_value_t = 0)]
    start_page: usize,
    /// sets a voice for the narration. see https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/index.html
    #[arg(long, default_value = "vits-piper-en_US-libritts_r-medium")]
    voice: String,
    /// sets the speed for the speaker
    #[arg(long, default_value_t = 1.0)]
    speed: f32,

    #[arg(long, default_value_t = true)]
    combine_pages: bool,

    /// for voices that have multiple speakers, pass a speaker_id.
    #[arg(short, long, default_value_t = 1)]
    speaker_id: i32,
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
    let stride_pixel: u32 = match (image.color()) {
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

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let pdf = PDF::from_file(args.input_file).unwrap();
    let page_images: Vec<DynamicImage> = pdf
        .render(
            pdf2image::Pages::Range(args.start_page as u32..=(pdf.page_count() - 1)),
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

    let mut dir = fs::read_dir(format!("tts/{}", args.voice)).expect("No TTS Model!");

    if dir.next().is_none() {
        return Err(anyhow!("Couldn't find tts model"));
    }
    let complete_pages: Vec<TtsAudio> = pages
        .par_iter()
        .map(move |page| {
            let config = VitsTtsConfig {
                model: "./tts/vits-piper-en_US-libritts_r-medium/en_US-libritts_r-medium.onnx"
                    .into(),
                data_dir: "./tts/vits-piper-en_US-libritts_r-medium/espeak-ng-data".into(),
                tokens: "./tts/vits-piper-en_US-libritts_r-medium/tokens.txt".into(),
                length_scale: 1.0,
                ..Default::default()
            };
            let mut tts = VitsTts::new(config);
            tts.create(page.contents.as_str(), args.speaker_id, args.speed)
                .unwrap()
        })
        .collect();
    let sample_rate = complete_pages[0].sample_rate;
    match args.combine_pages {
        true => sherpa_rs::write_audio_file(
            format!("out/{}.wav", args.output_file).as_str(),
            &complete_pages
                .iter()
                .map(|item| item.samples.clone())
                .reduce(|mut acc, page| {
                    acc.extend(page);
                    acc
                })
                .unwrap(),
            sample_rate,
        )
        .unwrap(),
        false => {
            for (i, page) in complete_pages.iter().enumerate() {
                sherpa_rs::write_audio_file(
                    format!("out/{}{}.wav", args.output_file, i).as_str(),
                    &page.samples,
                    sample_rate,
                )
                .unwrap();
            }
        }
    }
    Ok(())
}
