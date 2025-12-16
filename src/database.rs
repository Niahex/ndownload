use anyhow::Result;
use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct Video {
    pub id: String,
    pub platform: String,
    pub channel: String,
    pub title: String,
    pub url: String,
    pub downloaded: bool,
    pub download_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Database {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS videos (
                id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                channel TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                downloaded INTEGER NOT NULL DEFAULT 0,
                download_path TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn add_video(&self, video: &Video) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO videos (id, platform, channel, title, url, downloaded, download_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &video.id,
                &video.platform,
                &video.channel,
                &video.title,
                &video.url,
                video.downloaded as i32,
                &video.download_path,
                video.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn mark_as_downloaded(&self, video_id: &str, download_path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE videos SET downloaded = 1, download_path = ?1 WHERE id = ?2",
            params![download_path, video_id],
        )?;
        Ok(())
    }

    pub fn get_all_videos(&self) -> Result<Vec<Video>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, platform, channel, title, url, downloaded, download_path, created_at FROM videos ORDER BY created_at DESC"
        )?;

        let videos = stmt.query_map([], |row| {
            Ok(Video {
                id: row.get(0)?,
                platform: row.get(1)?,
                channel: row.get(2)?,
                title: row.get(3)?,
                url: row.get(4)?,
                downloaded: row.get::<_, i32>(5)? != 0,
                download_path: row.get(6)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(videos)
    }

    pub fn is_video_known(&self, video_id: &str) -> Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM videos WHERE id = ?1",
            params![video_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}
