use std::process::Command;

pub enum TtsEngine {
    Piper(String), // path to model
    Espeak,
    SpeechDispatcher,
}

pub struct TtsService {
    engine: TtsEngine,
}

impl TtsService {
    pub fn new(engine: TtsEngine) -> Self {
        Self { engine }
    }

    pub fn speak(&self, text: &str) -> Result<(), TtsError> {
        match &self.engine {
            TtsEngine::Piper(model_path) => {
                // Piper - локальный нейросетевой TTS
                let output = Command::new("piper")
                    .args(&["--model", model_path, "--output_file", "/tmp/frog_tts.wav"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()?;

                // Write text to stdin
                // Play audio via libcanberra or gstreamer
                todo!()
            }
            TtsEngine::Espeak => {
                Command::new("espeak-ng").arg(text).spawn()?;
                Ok(())
            }
            TtsEngine::SpeechDispatcher => {
                // spd-say
                Command::new("spd-say").arg(text).spawn()?;
                Ok(())
            }
        }
    }
}
