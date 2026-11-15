use config::{Config as ConfigEngine, ConfigError, Environment, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub feed_url: String,
    pub output_dir: PathBuf,
    /// TTS engine: "kokoro" for Kokoro TTS, "say" for macOS say command
    pub tts_engine: String,
    pub voice: String,
    pub speed: u32,
    pub interval_secs: u64,
    pub retention_days: u64,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let builder = ConfigEngine::builder()
            .set_default("feed_url", "https://guyanachronicle.com/feed/")?
            .set_default("output_dir", "./output")?
            .set_default("tts_engine", "kokoro")?
            .set_default("voice", "af_heart")?
            .set_default("speed", 175)?
            .set_default("interval_secs", 3600)?
            .set_default("retention_days", 5)?
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::with_prefix("NEWS"));

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
