use gpui::*;
use gpui::prelude::*;

#[derive(IntoElement)]
pub struct ProgressBar {
    progress: f32, // 0.0 to 1.0
    label: Option<SharedString>,
}

impl ProgressBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .text_size(px(12.0))
                        .text_color(rgb(0xd8dee9)) // NORD4
                        .child(label),
                )
            })
            .child(
                div()
                    .h(px(8.0))
                    .w_full()
                    .bg(rgb(0x3b4252)) // NORD1
                    .rounded_md()
                    .overflow_hidden()
                    .child(
                        div()
                            .h_full()
                            .w(relative(self.progress))
                            .bg(rgb(0x88c0d0)) // NORD8
                            .rounded_md(),
                    ),
            )
    }
}
