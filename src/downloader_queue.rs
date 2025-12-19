use anyhow::Result;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::AppContext;
use parking_lot::Mutex;
use smol::io::{AsyncBufReadExt, BufReader};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub video_id: String,
    pub video_url: String,
    pub title: String,
    pub output_path: PathBuf,
    pub status: DownloadStatus,
    pub progress: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Completed,
    Failed(String),
}

pub struct DownloadQueue {
    tasks: Arc<Mutex<Vec<DownloadTask>>>,
    tx: mpsc::UnboundedSender<DownloadTask>,
}

impl DownloadQueue {
    pub fn new(cx: &impl AppContext) -> Self {
        let tasks: Arc<Mutex<Vec<DownloadTask>>> = Arc::new(Mutex::new(Vec::new()));
        let (tx, mut rx) = mpsc::unbounded::<DownloadTask>();

        let tasks_clone = tasks.clone();

        // Worker qui traite les téléchargements
        cx.background_spawn(async move {
            while let Some(mut task) = rx.next().await {
                tracing::info!("Début du téléchargement: {}", task.title);

                // Mettre à jour le statut
                task.status = DownloadStatus::Downloading;
                {
                    let mut tasks_lock = tasks_clone.lock();
                    if let Some(t) = tasks_lock.iter_mut().find(|t| t.video_id == task.video_id) {
                        t.status = DownloadStatus::Downloading;
                    }
                }

                // Télécharger la vidéo avec mise à jour de progression
                let tasks_for_progress = tasks_clone.clone();
                let video_id = task.video_id.clone();

                match Self::download_video(&task, move |progress, speed, eta| {
                    let mut tasks_lock = tasks_for_progress.lock();
                    if let Some(t) = tasks_lock.iter_mut().find(|t| t.video_id == video_id) {
                        t.progress = progress;
                        t.speed = speed;
                        t.eta = eta;
                    }
                })
                .await
                {
                    Ok(_) => {
                        tracing::info!("Téléchargement terminé: {}", task.title);
                        task.status = DownloadStatus::Completed;
                        task.progress = 1.0;
                    }
                    Err(e) => {
                        tracing::error!("Erreur téléchargement {}: {}", task.title, e);
                        task.status = DownloadStatus::Failed(e.to_string());
                    }
                }

                // Mettre à jour le statut final
                {
                    let mut tasks_lock = tasks_clone.lock();
                    if let Some(t) = tasks_lock.iter_mut().find(|t| t.video_id == task.video_id) {
                        t.status = task.status.clone();
                        t.progress = task.progress;
                    }
                }
            }
        })
        .detach();

        Self { tasks, tx }
    }

    /// Ajoute une tâche de téléchargement à la queue
    pub async fn add_download(
        &self,
        video_id: String,
        video_url: String,
        title: String,
        output_path: PathBuf,
    ) -> Result<()> {
        let task = DownloadTask {
            video_id: video_id.clone(),
            video_url,
            title,
            output_path,
            status: DownloadStatus::Queued,
            progress: 0.0,
            speed: None,
            eta: None,
        };

        // Ajouter à la liste
        {
            let mut tasks = self.tasks.lock();
            tasks.push(task.clone());
        }

        // Envoyer au worker
        self.tx.clone().send(task).await?;

        Ok(())
    }

    /// Obtient la liste de toutes les tâches
    pub fn get_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.lock();
        tasks.clone()
    }

    /// Télécharge une vidéo avec yt-dlp
    async fn download_video<F>(task: &DownloadTask, mut on_progress: F) -> Result<()>
    where
        F: FnMut(f32, Option<String>, Option<String>) + Send + 'static,
    {
        // Créer le dossier de sortie si nécessaire
        if let Some(parent) = task.output_path.parent() {
            smol::fs::create_dir_all(parent).await?;
        }

        let output_template = task.output_path.to_string_lossy().to_string();

        let mut child = smol::process::Command::new("yt-dlp")
            .arg("--newline")
            .arg("-o")
            .arg(&output_template)
            .arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]")
            .arg(&task.video_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Lire la sortie pour extraire la progression
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Some(line) = lines.next().await {
                if let Ok(line) = line {
                    let (progress, speed, eta) = Self::parse_download_line(&line);
                    if let Some(p) = progress {
                        on_progress(p, speed, eta);
                    }
                }
            }
        }

        let status = child.status().await?;
        if !status.success() {
            anyhow::bail!("yt-dlp a échoué");
        }

        Ok(())
    }

    fn parse_download_line(line: &str) -> (Option<f32>, Option<String>, Option<String>) {
        if !line.contains("[download]") {
            return (None, None, None);
        }

        let mut progress = None;
        let mut speed = None;
        let mut eta = None;

        // Parse progress: "[download]  45.2% of ..."
        if line.contains('%') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if part.ends_with('%') {
                    if let Ok(val) = part.trim_end_matches('%').parse::<f32>() {
                        progress = Some(val / 100.0);
                    }
                }
                // Parse speed: "at 2.5MiB/s"
                if *part == "at" && i + 1 < parts.len() {
                    speed = Some(parts[i + 1].to_string());
                }
                // Parse ETA: "ETA 05:30"
                if *part == "ETA" && i + 1 < parts.len() {
                    eta = Some(parts[i + 1].to_string());
                }
            }
        }

        (progress, speed, eta)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let line = "[download]  45.2% of 100.00MiB at 2.5MiB/s ETA 05:30";
        let (progress, speed, eta) = DownloadQueue::parse_download_line(line);

        assert!(progress.is_some());
        assert!((progress.unwrap() - 0.452).abs() < 0.001);
        assert_eq!(speed, Some("2.5MiB/s".to_string()));
        assert_eq!(eta, Some("05:30".to_string()));
    }

    #[test]
    fn test_parse_progress_no_speed() {
        let line = "[download]  75.0% of 100.00MiB";
        let (progress, speed, eta) = DownloadQueue::parse_download_line(line);

        assert_eq!(progress, Some(0.75));
        assert_eq!(speed, None);
        assert_eq!(eta, None);
    }

    #[test]
    fn test_parse_non_download_line() {
        let line = "[info] Downloading video...";
        let (progress, speed, eta) = DownloadQueue::parse_download_line(line);

        assert_eq!(progress, None);
        assert_eq!(speed, None);
        assert_eq!(eta, None);
    }
}
