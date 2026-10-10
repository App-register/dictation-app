use whisper_cpp_plus::{WhisperContext, FullParams, SamplingStrategy};
use crate::Result;
use crate::DictoError;

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

    /// Ensure the model is loaded (lazy).
    fn ensure_loaded(&mut self) -> Result<()> {
        if self.ctx.is_none() {
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

        let mut params = FullParams::new(SamplingStrategy::Greedy);

        // Set language if not auto
        if self.language != "auto" {
            params.set_language(&self.language);
        }

        // Speed optimizations:
        params.set_n_threads(num_cpus::get() as i32);
        params.set_suppress_non_speech_tokens(true);
        // Don't compute timestamps — faster
        // (timestamps disabled by default in Greedy mode)

        let text = ctx
            .transcribe_with_params(audio, params)
            .map_err(|e| DictoError::Stt(format!("Transcription failed: {}", e)))?;

        Ok(text.trim().to_string())
    }

    /// Get the actual model path (resolves "tiny"/"base" to real path).
    pub fn resolve_model_path(requested: &str) -> String {
        match requested {
            "tiny" => Self::download_if_not_present("ggml-tiny.bin", 
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"),
            "base" => Self::download_if_not_present("ggml-base.bin",
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"),
            path => path.to_string(),
        }
    }

    fn download_if_not_present(filename: &str, url: &str) -> String {
        let models_dir = crate::config::Config::config_dir().join("models");
        let model_path = models_dir.join(filename);

        if !model_path.exists() {
            log::info!("Model not found at {:?}, downloading...", model_path);
            // Download would happen here (ureq or reqwest)
            // For now, return the expected path and let the user download
            log::warn!("Please download: {} → {:?}", url, model_path);
        }

        model_path.to_string_lossy().to_string()
    }
}

fn num_cpus() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }