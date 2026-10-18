use whisper_cpp_plus::{WhisperContext, TranscriptionParams};
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

        // Builder pattern — the correct API for whisper-cpp-plus
        let mut builder = TranscriptionParams::builder()
            .n_threads(num_cpus() as i32)
            .disable_timestamps()
            .single_segment(true);

        // Set language if not auto
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
        match requested {
            "tiny" => Self::model_path_for("ggml-tiny.bin"),
            "base" => Self::model_path_for("ggml-base.bin"),
            path => path.to_string(),
        }
    }

    fn model_path_for(filename: &str) -> String {
        crate::config::Config::config_dir()
            .join("models")
            .join(filename)
            .to_string_lossy()
            .to_string()
    }
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}