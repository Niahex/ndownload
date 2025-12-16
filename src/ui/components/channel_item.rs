use gpui::*;

use super::super::{Channel, Platform, NORD6, NORD11, NORD15};

#[derive(IntoElement)]
pub struct ChannelItem {
    channel: Channel,
}

impl ChannelItem {
    pub fn new(channel: Channel) -> Self {
        Self { channel }
    }
}

impl RenderOnce for ChannelItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let platform_color = match self.channel.platform {
            Platform::YouTube => rgb(NORD11),
            Platform::Twitch => rgb(NORD15),
        };
        let platform_name = match self.channel.platform {
            Platform::YouTube => "YouTube",
            Platform::Twitch => "Twitch",
        };

        div()
            .flex()
            .items_center()
            .gap_3()
            .child(
                div()
                    .px_2()
                    .py_1()
                    .bg(platform_color)
                    .rounded_sm()
                    .child(
                        div()
                            .text_color(rgb(NORD6))
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .child(platform_name),
                    ),
            )
            .child(
                div()
                    .text_color(rgb(NORD6))
                    .text_size(px(14.0))
                    .child(self.channel.name.clone()),
            )
    }
}
