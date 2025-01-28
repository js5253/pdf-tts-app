#![feature(slice_as_array)]

use image::{EncodableLayout, GenericImageView, GrayImage, Luma, imageops::FilterType};
fn find_black_line(img: &GrayImage) -> Option<u32> {
    let (width, height) = img.dimensions();

    // Threshold to identify black pixels (adjust as needed)
    let black_threshold: u8 = 30;

    // Find the column with the most black pixels
    let mut max_black_pixels = 0;
    let mut split_column = width / 2; // Default to the middle if no line is found
    let min_location = width / 5;
    for x in min_location..width {
        let mut black_pixels = 0;

        for y in 0..height {
            let pixel = img.get_pixel(x, y).0[0];
            if pixel < black_threshold {
                black_pixels += 1;
            }
        }

        // Update the split column if this column has more black pixels
        if black_pixels > max_black_pixels {
            max_black_pixels = black_pixels;
            split_column = x;
        }
    }

    // Only return the column if a significant black line is found
    if max_black_pixels > height / 2 {
        Some(split_column)
    } else {
        None
    }
}

fn split_image_at_column(img: &mut GrayImage, column: u32) -> (GrayImage, GrayImage) {
    let (width, height) = img.dimensions();

    let left_page = image::imageops::crop(img, 0, 0, column, height).to_image();
    let right_page = image::imageops::crop(img, column, 0, width - column, height).to_image();

    (left_page, right_page)
}

fn main() {
    // Load the image and convert to grayscale
    let mut img = image::open("input/1.png")
        .expect("Failed to open image")
        .to_luma8();

    // Find the black line
    if let Some(split_column) = find_black_line(&img) {
        println!("Detected black line at column: {}", split_column);

        // Split the image at the detected column
        let (left_page, right_page) = split_image_at_column(&mut img, split_column);

        // Save the split pages
        left_page
            .save("left_page.png")
            .expect("Failed to save left page");
        right_page
            .save("right_page.png")
            .expect("Failed to save right page");

        let mut contents = tesseract::ocr("1.png", "eng").unwrap();
        clean_up_text(&mut contents);
        println!("{:?}", contents);
    } else {
        println!("No black line detected. Splitting at the middle.");
        let (left_page, right_page) = split_image_at_column(&mut img.clone(), img.width() / 2);

        left_page
            .save("left_page.png")
            .expect("Failed to save left page");
        right_page
            .save("right_page.png")
            .expect("Failed to save right page");
    }
}

fn clean_up_text(text: &mut String) {
    *text = text.replace("\n", " ");
}
