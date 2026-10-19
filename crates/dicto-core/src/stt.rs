use whisper_cpp_plus::{WhisperContext, TranscriptionParams};
use crate::Result;
use crate::DictoError;
use std::path::{Path, PathBuf};
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
            let model_path = Path::new(&self.model_path);
            if !model_path.exists() {
                let filename = model_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("model.bin");
                download_model(filename, &self.model_path)?;
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

    /// Get the actual model path (resolves "tiny"/"base" to real path).
    pub fn resolve_model_path(requested: &str) -> String {
        let dir = crate::config::Config::config_dir().join("models");
        let filename = match requested {
            "tiny" => "ggml-tiny.bin",
            "base" => "ggml-base.bin",
            _ => return requested.to_string(),
        };
        dir.join(filename).to_string_lossy().to_string()
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Download a Whisper model from HuggingFace if it doesn't exist locally.
fn download_model(filename: &str, dest: &str) -> Result<()> {
    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    );
    let dest_path = PathBuf::from(dest);

    // Ensure parent directory exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| DictoError::Stt(format!("Failed to create model dir: {}", e)))?;
    }

    log::info!("Downloading model: {} (~142 MB)", filename);
    log::info!("From: {}", url);

    let resp = ureq::get(&url)
        .call()
        .map_err(|e| DictoError::Stt(format!("Download failed: {}", e)))?;

    let mut body: Vec<u8> = Vec::new();
    resp.into_reader()
        .read_to_end(&mut body)
        .map_err(|e| DictoError::Stt(format!("Download read error: {}", e)))?;

    std::fs::write(&dest_path, &body)
        .map_err(|e| DictoError::Stt(format!("Failed to write model: {}", e)))?;

    log::info!("Model downloaded ({} MB)", body.len() / 1_048_576);
    Ok(())
}