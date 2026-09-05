// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Builder, Manager};
fn setup(app: &AppHandle) {
    // app.
}

fn main() {
    // Builder::default().setup(move |app| tauri::async_runtime::block_on(setup(app.handle())));
 pdf_tts_app_lib::run();
}
