use anyhow::{anyhow, Context};
use glam::UVec2;
use image::{ColorType, DynamicImage};
use markdown_strip::strip_markdown;
use pdf_inspector::process_pdf;
use pdfium_render::prelude::*;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
};

use crate::util::CommandResult;

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

fn text_from_image(image: &DynamicImage) -> CommandResult<String> {
    let image_dims = UVec2::from_array([image.width(), image.height()]);
    let coords = UVec2::ZERO;

    Ok(ocr_region(image_dims, coords, image)?)
}

fn run_pdf_text(input_file: &PathBuf) -> CommandResult<Vec<Page>> {
    let mut p: Vec<Page> = Vec::new();
    let input_path = Path::new(&input_file);
    let pdf = process_pdf(input_path);

    if let Some(text) = &pdf
        .map_err(|_| anyhow!("Could not extract text from PDF."))?
        .markdown
    {
        p.push(Page {
            contents: strip_markdown(text),
            index: 0,
        })
    };
    Ok(p)
}
fn run_pdf_ocr(
    input_file: &PathBuf,
    _start_page: u32,
    range: Option<[[u32; 2]; 2]>,
) -> CommandResult<Vec<Page>> {
    let pdfium = Pdfium::default();
    let file = pdfium.load_pdf_from_file(input_file, None)?;
    let mut config = PdfRenderConfig::default();
    if let Some(range) = range {
        config = config.clip( range[0][0] as i32, range[0][1] as i32, range[1][0] as i32, range[1][1] as i32);
    }
    let page_images: Vec<PdfBitmap<'_>> = file
        .pages()
        .iter()
        .map(|page| {
            let width = page.width().value;
            let height = page.height().value;
            let mut bitmap =
                PdfBitmap::empty(width as Pixels, height as Pixels, PdfBitmapFormat::Gray)
                    .map_err(|_| anyhow!("Failed to render PDF to an image"))
                    .unwrap();
            let _ = page.render_into_bitmap_with_config(&mut bitmap, &config);
            bitmap
        })
        .collect();

    let pages: Vec<Page> = page_images
        .par_iter()
        .enumerate()
        .filter_map(|(idx, image)| {
            Some(Page {
                index: idx as u32,
                contents: text_from_image(&image.as_image().map_err(|_| anyhow!("ABC")).ok()?).unwrap(),
            })
        })
        .collect();
    Ok(pages)
}
pub fn get_page_contents(
    input_file: &PathBuf,
    use_ocr: bool,
    start_page: u32,
    range: Option<[[u32; 2]; 2]>,
) -> CommandResult<Vec<Page>> {
    match input_file
        .extension()
        .ok_or(anyhow!("No File Extension Found"))?
        == OsStr::new("pdf")
    {
        true => match use_ocr {
            true => run_pdf_ocr(input_file, start_page, range),
            false => run_pdf_text(input_file),
        },
        false => {
            let contents = anydoc::to_markdown(input_file)
                .map_err(|_| anyhow!("could not get markdown from PDF"))?;
            let contents = strip_markdown(&contents.to_string());
            Ok(vec![Page { index: 0, contents }])
        }
    }
    // post-processing for either anydoc or the OCR'ed/Tesseracted system goes here.
}
