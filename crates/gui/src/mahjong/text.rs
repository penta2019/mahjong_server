use bevy::prelude::*;
use mahjong_core::model::*;

use super::param::param;

pub fn create_text(text: String, font_size: f32) -> impl Bundle {
    let font = param().asset_server.load("font/NotoSerifCJKjp-Regular.otf");
    (
        Text::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(Color::WHITE),
        // TextShadow::default(),
    )
}

pub fn wind_to_char(ti: Tnum) -> char {
    ['?', '東', '南', '西', '北'][ti]
}
