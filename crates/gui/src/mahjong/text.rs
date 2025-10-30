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

pub fn create_text_with_color(text: String, font_size: f32, color: Color) -> impl Bundle {
    let font = param().asset_server.load("font/NotoSerifCJKjp-Regular.otf");
    (
        Text::new(text),
        TextFont {
            font,
            font_size,
            ..default()
        },
        TextColor(color),
        // TextShadow::default(),
    )
}

pub fn wind_to_char_jp(ti: Tnum) -> char {
    ['?', '東', '南', '西', '北'][ti]
}

pub fn round_string(round: usize, dealer: Seat, honba: Option<usize>) -> String {
    let honba_str = if let Some(h) = honba {
        format!(" {h}本場")
    } else {
        "".into()
    };
    format!(
        "{}{}局{}",
        wind_to_char_jp(round % 4 + 1),
        dealer + 1,
        honba_str
    )
}
