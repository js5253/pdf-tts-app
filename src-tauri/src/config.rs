use std::{path::Path, fs};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, specta::Type)]
pub struct TtsJobConfig {
    pub output_prefix: String,
    pub output_dir: String,
    pub start_page: u32,
    /// sets a voice for the narration. see https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/index.html
    pub voice: String,
    pub speed: f32,
    pub combine_pages: bool,
    /// for voices that have multiple speakers, pass a speaker_id.
    pub speaker_id: u32,
    pub use_ocr: bool,
    // end TTS config here
    pub input_file: String,
}

#[derive(Serialize, Deserialize, Clone, specta::Type)]
pub struct TtsAppConfig {
    /// start the narration at a certain page
    pub output_dir: String,
    pub start_page: u32,
    pub output_prefix: String,
    pub voice: String,
    pub speed: f32,
    pub combine_pages: bool,
    pub speaker_id: u32,
}

impl TtsAppConfig {
    pub fn load_or_default(path: &Path) -> Self {
        let path = Path::new("config.toml");
        let data_dir = path.to_str().unwrap().to_string();
        if fs::exists(path).is_ok_and(|item| item) {
            let config = fs::read_to_string("config.toml").unwrap();
            let config: TtsAppConfig = toml::from_str(config.as_str()).unwrap();
            // later, add code to handle json file updates 
            config
        } else {
            let config = TtsAppConfig {
                start_page: 0,
                output_dir: data_dir,
                voice: String::from("vits-piper-en_US-libritts_r-medium"),
                speed: 1.0,
                combine_pages: true,
                speaker_id: 1,
                output_prefix: String::from("page_"),
            };
            fs::write(path, toml::to_string(&config).unwrap()).unwrap();

            config
        }
    }
}

