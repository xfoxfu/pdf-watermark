use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct UtilsSettings {
    pub mark_pdf_max_size_byte: usize,
    pub mark_pdf_timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub bind_address: String,
    pub utils: UtilsSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::from_str(
                include_str!("../assets/config/settings.default.toml"),
                FileFormat::Toml,
            ))
            .add_source(File::with_name(&format!("settings.{}", run_mode)).required(false))
            .add_source(File::with_name("settings.local").required(false))
            .add_source(Environment::with_prefix("app"))
            .build()?;

        s.try_deserialize()
    }
}
