use image::{DynamicImage, imageops};
use pdf2image::{PDF, RenderOptionsBuilder};
use rayon::prelude::*;
use std::{
    env,
    fmt::{Display, Error, Formatter},
    fs::{self, DirEntry, File},
    io::Read,
};
use tts_app::extract_number;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDF file to open
    #[arg(short, long)]
    input_file: String,
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
    let stride_pixel: usize = 1;
    let stride_line: usize = image_width;
    let width = dims.0;
    let height = dims.1;
    let x = coords.1;
    let y = coords.0;
    let byte_offset: usize = (y * stride_line + y * stride_pixel) as usize;
    assert!(x + width <= image_width);
    assert!(y + height <= image_width);
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
    println!("page size {}x{}", image.width(), image.height());
    //
    //let dims: (usize, usize) = (1166 * 1614 / 3099, 1809 * 1239 / 1614); //TODO
    //let left_coords: (usize, usize) = (374 * 1614 / 3099, 193 * 1239 / 1614); //TODO
    //let right_coords: (usize, usize) = (1808 * 1614 / 3099, 196 * 1239 / 1614); //TODO
    let image_width: usize = image.width() as usize;
    let image_height: usize = image.height() as usize;
    let dims: (usize, usize) = (1166 * image_width / 3099, 1809 * image_height / 2379); //TODO
    let left_coords: (usize, usize) = (374 * image_width / 3099, 193 * image_height / 2379); //TODO
    let right_coords: (usize, usize) = (1808 * image_width / 3099, 196 * image_height / 2379); //TODO

    let mut left_text = ocr_region(dims, left_coords, &image);
    let mut right_text = ocr_region(dims, right_coords, &image);
    post_process_text(&mut left_text);
    post_process_text(&mut right_text);
    (left_text, right_text)
}
///FIXME: THIS FUNCTION IS SHIT
fn post_process_text(string: &mut String) {
    *string = string
        .replace("- ", "")
        .replace("- ", "")
        .replace("\n", " ");
}

fn main() -> Result<(), anyhow::Error> {
    println!("{:?}", env::current_dir().unwrap());
    let args = Args::parse();

    let pdf = PDF::from_file(args.input_file).unwrap();
    let mut page_images: Vec<DynamicImage> = pdf
        .render(
            pdf2image::Pages::All,
            RenderOptionsBuilder::default().build()?,
        )?
        .iter_mut()
        .map(|page| page.rotate90())
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
    println!("{}", pages.get(0).unwrap());
    Ok(())
}
