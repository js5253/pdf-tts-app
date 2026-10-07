// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![deny(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#[tokio::main]
async fn main() {
    pdf_tts_app_lib::run();
}
