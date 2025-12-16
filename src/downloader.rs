use anyhow::Result;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

pub struct Downloader {
    output_dir: PathBuf,
}

impl Downloader {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Télécharge une vidéo avec yt-dlp
    pub async fn download_video(&self, url: &str, channel: &str) -> Result<PathBuf> {
        let channel_dir = self.output_dir.join(channel);
        tokio::fs::create_dir_all(&channel_dir).await?;

        tracing::info!("Téléchargement de {} dans {:?}", url, channel_dir);

        let output = Command::new("yt-dlp")
            .arg("--no-playlist")
            .arg("--output")
            .arg(format!("{}/%(title)s.%(ext)s", channel_dir.display()))
            .arg("--print")
            .arg("after_move:filepath")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yt-dlp a échoué: {}", error);
        }

        let filepath = String::from_utf8(output.stdout)?
            .trim()
            .to_string();

        Ok(PathBuf::from(filepath))
    }

    /// Obtient les informations d'une vidéo sans la télécharger
    pub async fn get_video_info(&self, url: &str) -> Result<VideoInfo> {
        let output = Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--no-playlist")
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yt-dlp a échoué: {}", error);
        }

        let json_str = String::from_utf8(output.stdout)?;
        let info: VideoInfo = serde_json::from_str(&json_str)?;

        Ok(info)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub uploader: Option<String>,
    pub upload_date: Option<String>,
}
