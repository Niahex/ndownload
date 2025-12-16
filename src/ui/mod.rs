use gpui::*;

pub struct NDownloadApp {
    // État de l'application
}

impl NDownloadApp {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {}
    }
}

impl Render for NDownloadApp {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x1e1e1e))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .size_full()
                    .child(
                        div()
                            .text_color(rgb(0xffffff))
                            .text_size(px(24.0))
                            .child("NDownloader")
                    )
                    .child(
                        div()
                            .text_color(rgb(0xaaaaaa))
                            .text_size(px(14.0))
                            .child("Automatic video downloader for Twitch and YouTube")
                    )
            )
    }
}
