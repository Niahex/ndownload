use gpui::prelude::FluentBuilder;
use gpui::*;

use super::super::{VideoInfo, VideoStatus, NORD13, NORD14, NORD6, NORD9};
use super::ProgressBar;

#[derive(IntoElement)]
pub struct VideoItem {
    video: VideoInfo,
    progress: Option<f32>,
}

impl VideoItem {
    pub fn new(video: VideoInfo) -> Self {
        Self {
            video,
            progress: None,
        }
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress);
        self
    }
}

impl RenderOnce for VideoItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (indicator_color, status_text, status_color) = match self.video.status {
            VideoStatus::Downloaded => (rgb(NORD14), "Téléchargé", rgb(NORD14)),
            VideoStatus::Downloading => (rgb(NORD9), "En cours...", rgb(NORD9)),
            VideoStatus::NotDownloaded => (rgb(NORD13), "Non téléchargé", rgb(NORD13)),
        };

        div()
            .flex()
            .items_center()
            .gap_3()
            .child(div().w_3().h_3().rounded_full().bg(indicator_color))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .flex_1()
                    .child(
                        div()
                            .text_color(rgb(NORD6))
                            .text_size(px(14.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.video.title.clone()),
                    )
                    .child(
                        div()
                            .text_color(status_color)
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(status_text),
                    )
                    .when(self.progress.is_some(), |this| {
                        this.child(
                            div()
                                .w_full()
                                .mt_1()
                                .child(ProgressBar::new(self.progress.unwrap())),
                        )
                    }),
            )
    }
}
