// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Builder, Manager};
fn setup(app: &AppHandle) {
    // app.
}
#[tokio::main]
async fn main() {
    // let _ = Builder::default().setup(move |app| {
    // setup(app.handle());
    // Ok(())});
    pdf_tts_app_lib::run();
}
