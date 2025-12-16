use anyhow::Result;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::AppContext;
use parking_lot::Mutex;
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

                // Télécharger la vidéo
                match Self::download_video(&task).await {
                    Ok(_) => {
                        tracing::info!("Téléchargement terminé: {}", task.title);
                        task.status = DownloadStatus::Completed;
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
                    }
                }
            }
        }).detach();

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
    async fn download_video(task: &DownloadTask) -> Result<()> {
        // Créer le dossier de sortie si nécessaire
        if let Some(parent) = task.output_path.parent() {
            smol::fs::create_dir_all(parent).await?;
        }

        let output_template = task.output_path.to_string_lossy().to_string();

        let output = smol::process::Command::new("yt-dlp")
            .arg("-o")
            .arg(&output_template)
            .arg("-f")
            .arg("bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]")
            .arg(&task.video_url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("yt-dlp a échoué: {}", error);
        }

        Ok(())
    }
}
