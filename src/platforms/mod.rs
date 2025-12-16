pub mod youtube;
pub mod twitch;

use anyhow::Result;
use crate::database::Video;

pub trait Platform {
    async fn get_latest_videos(&self, channel: &str) -> Result<Vec<Video>>;
}
