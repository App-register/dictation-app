use whisper_cpp_plus::{WhisperContext, TranscriptionParams};
use crate::Result;
use crate::DictoError;
use std::io::Read;

/// STT engine wrapping whisper-cpp-plus.
/// Model is loaded lazily on first use, not at startup.
pub struct SttEngine {
    model_path: String,
    language: String,
    ctx: Option<WhisperContext>,
}

impl SttEngine {
    pub fn new(model_path: &str, language: &str) -> Self {
        Self {
            model_path: model_path.to_string(),
            language: language.to_string(),
            ctx: None,
        }
    }

    /// Ensure the model is loaded (lazy). Auto-downloads if missing.
    fn ensure_loaded(&mut self) -> Result<()> {
        if self.ctx.is_none() {
            // Auto-download model if missing
            if !model_exists(&self.model_path) && !std::path::Path::new(&self.model_path).exists() {
                let name = if self.model_path.contains("tiny") { "tiny" } else { "base" };
                download_model_file(name)
                    .map_err(|e| DictoError::Stt(e))?;
            }

            log::info!("Loading model: {}", self.model_path);
            self.ctx = Some(
                WhisperContext::new(&self.model_path)
                    .map_err(|e| DictoError::Stt(format!("Failed to load model: {}", e)))?
            );
            log::info!("Model loaded successfully");
        }
        Ok(())
    }

    /// Transcribe a full audio buffer (16kHz mono f32).
    /// Returns the transcribed text.
    pub fn transcribe(&mut self, audio: &[f32]) -> Result<String> {
        self.ensure_loaded()?;
        let ctx = self.ctx.as_ref().unwrap();

        let mut builder = TranscriptionParams::builder()
            .n_threads(num_cpus() as i32)
            .disable_timestamps()
            .single_segment(true);

        if self.language != "auto" {
            builder = builder.language(self.language.as_str());
        }

        let params = builder.build();

        let result = ctx
            .transcribe_with_params(audio, params)
            .map_err(|e| DictoError::Stt(format!("Transcription failed: {}", e)))?;

        Ok(result.text.trim().to_string())
    }
    }

    /// Get the filesystem path for a named model ("tiny" or "base").
pub fn get_model_path(size: &str) -> String {
    let dir = crate::config::Config::config_dir().join("models");
    let filename = match size {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        _ => return size.to_string(),
    };
    dir.join(filename).to_string_lossy().to_string()
}

/// Download a Whisper model from HuggingFace. Shows progress via log.
pub fn download_model_file(size: &str) -> std::result::Result<(), String> {
    let filename = match size {
        "tiny" => "ggml-tiny.bin",
        "base" => "ggml-base.bin",
        _ => return Err(format!("Unknown model: {}", size)),
    };
    let dest = get_model_path(size);
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    );
    let dest_path = std::path::PathBuf::from(&dest);

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create model dir: {}", e))?;
    }

    log::info!("Downloading {} from {} …", filename, url);
    let resp = ureq::get(&url)
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let mut body: Vec<u8> = Vec::new();
    resp.into_reader()
        .read_to_end(&mut body)
        .map_err(|e| format!("Download error: {}", e))?;

    std::fs::write(&dest_path, &body)
        .map_err(|e| format!("Failed to write model: {}", e))?;

    let mb = body.len() as f64 / 1_048_576.0;
    log::info!("Downloaded {} ({:.1} MB)", filename, mb);
    Ok(())
}

/// Check if a named model file exists on disk.
pub fn model_exists(size: &str) -> bool {
    std::path::Path::new(&get_model_path(size)).exists()
}
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}