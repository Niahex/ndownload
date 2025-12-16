use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Répertoire de téléchargement
    pub download_dir: PathBuf,

    /// Chaînes YouTube à surveiller
    pub youtube_channels: Vec<String>,

    /// Chaînes Twitch à surveiller
    pub twitch_channels: Vec<String>,

    /// Intervalle de vérification en minutes
    pub check_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::from("./downloads"),
            youtube_channels: Vec::new(),
            twitch_channels: Vec::new(),
            check_interval: 60,
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
