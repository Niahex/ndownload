use gpui::*;

#[derive(IntoElement)]
pub struct ProgressBar {
    progress: f32, // 0.0 to 1.0
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
        }
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .w_full()
            .h(px(4.0))
            .bg(rgb(0x3b4252))
            .rounded(px(2.0))
            .child(
                div()
                    .h_full()
                    .w(relative(self.progress))
                    .bg(rgb(0x88c0d0))
                    .rounded(px(2.0))
            )
    }
}
