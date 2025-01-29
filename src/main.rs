use image::{ColorType, DynamicImage, imageops};
use pdf2image::{PDF, RenderOptionsBuilder};
use rayon::prelude::*;
use sherpa_rs::tts::{VitsTts, VitsTtsConfig};
use std::{
    env,
    fmt::{Display, Error, Formatter},
    fs::{self, DirEntry, File},
    io::Read,
};
use tts_app::extract_number;

use clap::Parser;

/// Program that allows you to use TTS from OCRed PDFs
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDF file to open
    #[arg(short, long)]
    input_file: String,
    /// start TTS at a certain page
    #[arg(long)]
    start_page: Option<u32>,
    /// sets a voice for the audiobook
    #[arg(long)]
    voice: Option<String>,
    /// sets the speed for the speaker
    #[arg(long)]
    speed: Option<f32>,
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

fn ocr_region(dims: (usize, usize), coords: (usize, usize), image: &DynamicImage) -> String {
    let image_width: usize = image.width() as usize;
    let image_height: usize = image.height() as usize;
    assert!(image.color() == ColorType::L8);
    let stride_pixel: usize = 1;
    let stride_line: usize = image_width * stride_pixel;
    let width = dims.0;
    let height = dims.1;
    let x = coords.0;
    let y = coords.1;
    let byte_offset: usize = (y * stride_line + x * stride_pixel) as usize;
    assert!(x + width <= image_width);
    assert!(y + height <= image_height);
    let text = tesseract::ocr_from_frame(
        &image.as_bytes()[byte_offset..],
        width as i32,
        height as i32,
        stride_pixel as i32,
        stride_line as i32,
        "eng",
    )
    .unwrap();
    text
}

fn text_from_image(image: &DynamicImage) -> (String, String) {
    // size 1166 1809
    //
    let image_width: usize = image.width() as usize;
    let image_height: usize = image.height() as usize;
    let dims: (usize, usize) = (1166 * image_width / 3099, 1809 * image_height / 2379); //TODO
    let left_coords: (usize, usize) = (374 * image_width / 3099, 193 * image_height / 2379); //TODO
    let right_coords: (usize, usize) = (1808 * image_width / 3099, 196 * image_height / 2379); //TODO

    let mut left_text = post_process_text(&ocr_region(dims, left_coords, &image));
    let mut right_text = post_process_text(&ocr_region(dims, right_coords, &image));
    (left_text, right_text)
}
///FIXME: THIS FUNCTION IS SHIT
fn post_process_text(string: &String) -> String {
    string.replace("-\n", "").replace("\n", " ")
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let pdf = PDF::from_file(args.input_file).unwrap();
    let mut page_images: Vec<DynamicImage> = pdf
        .render(
            pdf2image::Pages::All,
            RenderOptionsBuilder::default().build()?,
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

    let config = VitsTtsConfig {
        model: "./tts/vits-piper-en_US-libritts_r-medium/en_US-libritts_r-medium.onnx".into(),
        data_dir: "./tts/vits-piper-en_US-libritts_r-medium/espeak-ng-data".into(),
        tokens: "./tts/vits-piper-en_US-libritts_r-medium/tokens.txt".into(),
        length_scale: 1.0,
        ..Default::default()
    };
    let mut tts = VitsTts::new(config);
    let sid = 0;
    fs::create_dir("out").unwrap();
    for page_index in args.start_page.unwrap_or(0)..=(pages.len() - 1) as u32 {
        let page = &pages.get(page_index as usize).unwrap().contents;
        let audio = tts
            .create(page.as_str(), sid, args.speed.unwrap_or(1.0))
            .unwrap();
        sherpa_rs::write_audio_file(
            format!("out/audio{}.wav", page_index).as_str(),
            &audio.samples,
            audio.sample_rate,
        )
        .unwrap();
        println!("Created audio.wav");
    }
    Ok(())
}
