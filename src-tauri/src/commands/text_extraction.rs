use anyhow::{anyhow, Context};
use glam::UVec2;
use image::{ColorType, DynamicImage};
use markdown_strip::strip_markdown;
use pdf2image::{RenderOptionsBuilder, PDF};
use pdf_inspector::process_pdf;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    path::Path,
};

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

fn ocr_region(dims: UVec2, coords: UVec2, image: &DynamicImage) -> anyhow::Result<String> {
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
    assert!(UVec2::cmple(coords + dims, image_dims).all());
    let text = tesseract::ocr_from_frame(
        image.as_bytes(),
        dims.x as i32,
        dims.y as i32,
        stride_pixel as i32,
        stride_line as i32,
        "eng",
    )
    .context("Failed to extract text from a PDF image")?;
    Ok(text)
}

fn text_from_image(image: &DynamicImage) -> String {
    let image_dims = UVec2::from_array([image.width(), image.height()]);
    let coords = UVec2::ZERO;

    ocr_region(image_dims, coords, image).unwrap()
}

fn run_pdf_text(input_file: &String) -> anyhow::Result<Vec<Page>> {
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
fn run_pdf_ocr(input_file: &String, start_page: u32) -> anyhow::Result<Vec<Page>> {
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
        .map(|(idx, image)| Page {
            index: idx as u32,
            contents: text_from_image(image),
        })
        .collect();
    Ok(pages)
}
pub fn get_page_contents(
    input_file: &String,
    use_ocr: bool,
    start_page: u32,
) -> anyhow::Result<Vec<Page>> {
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
