use anyhow::{Context, anyhow};
use image::{ColorType, DynamicImage};
use glam::UVec2;
use markdown_strip::strip_markdown;
use pdf_inspector::process_pdf;
use pdf2image::{PDF, RenderOptionsBuilder};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{fmt::{self, Display, Formatter}, path::Path};

use crate::CommandResult;
#[derive(Debug)]
pub struct Page {
    pub index: u32,
    pub contents: String,
}
impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Page {}: {}", self.index, self.contents)
    }
}

fn ocr_region(dims: UVec2, coords: UVec2, image: &DynamicImage) -> CommandResult<String> {
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



fn run_pdf_text(input_file: &String) -> CommandResult<Vec<Page>> {
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
fn run_pdf_ocr(input_file: &String, start_page: u32) -> CommandResult<Vec<Page>> {
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
pub fn get_page_contents(input_file: &String, use_ocr: bool, start_page: u32) -> CommandResult<Vec<Page>> {
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