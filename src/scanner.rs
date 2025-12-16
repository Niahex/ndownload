use anyhow::Result;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub upload_date: Option<String>,
    #[serde(default)]
    pub uploader: Option<String>,
}

pub struct VideoScanner {
    storage_paths: Vec<String>,
}

impl VideoScanner {
    pub fn new() -> Self {
        Self {
            storage_paths: vec![
                "/run/mount/ve_stock_1".to_string(),
                "/run/mount/ve_stock_2".to_string(),
                "/run/mount/ve_ext_1".to_string(),
            ],
        }
    }

    /// Scanne les vidéos disponibles d'une chaîne avec yt-dlp
    pub async fn scan_channel_videos(&self, channel_url: &str) -> Result<Vec<VideoMetadata>> {
        tracing::info!("Scan des vidéos de: {}", channel_url);

        let output = Command::new("yt-dlp")
            .arg("--flat-playlist")
            .arg("--dump-json")
            .arg("--playlist-end")
            .arg("50") // Limiter à 50 vidéos pour le moment
            .arg(channel_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yt-dlp a échoué: {}", error);
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut videos = Vec::new();

        // Chaque ligne est un JSON
        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<VideoMetadata>(line) {
                Ok(video) => videos.push(video),
                Err(e) => {
                    tracing::warn!("Erreur parsing JSON: {} - ligne: {}", e, line);
                }
            }
        }

        tracing::info!("Trouvé {} vidéos", videos.len());
        Ok(videos)
    }

    /// Vérifie si une vidéo est déjà téléchargée en comparant la durée
    pub fn is_video_downloaded(&self, channel_name: &str, duration: Option<f64>) -> Option<String> {
        let Some(target_duration) = duration else {
            return None;
        };

        for storage_path in &self.storage_paths {
            let channel_path = format!("{}/{}", storage_path, channel_name);

            // Vérifier si le dossier existe
            if let Ok(entries) = std::fs::read_dir(&channel_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_file() {
                        continue;
                    }

                    // Vérifier la durée de la vidéo locale
                    if let Some(local_duration) = Self::get_video_duration(&path) {
                        // Tolérance de 5 secondes
                        if (local_duration - target_duration).abs() < 5.0 {
                            return Some(path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Obtient la durée d'une vidéo locale avec ffprobe
    fn get_video_duration(_path: &std::path::Path) -> Option<f64> {
        // Utiliser ffprobe pour obtenir la durée
        // Pour simplifier, on retourne None pour l'instant
        // TODO: Implémenter avec ffprobe
        None
    }

    /// Trouve le meilleur disque de stockage (celui avec le plus d'espace)
    pub fn find_best_storage_path(&self) -> Result<String> {
        // Pour l'instant, retourner le premier disponible
        for path in &self.storage_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        anyhow::bail!("Aucun disque de stockage disponible")
    }
}
