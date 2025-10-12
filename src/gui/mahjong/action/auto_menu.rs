use bevy::prelude::*;

use super::{BUTTON_INACTIVE, GameButton};
use crate::gui::mahjong::param::param;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutoButton {
    Discard,
    Sort,
    Win,
    Skip,
}

pub fn create_auto_menu() -> Entity {
    let bundle = (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            bottom: Val::Percent(0.0),
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            create_auto_button(AutoButton::Discard, "Discard"),
            create_auto_button(AutoButton::Sort, "Sort"),
            create_auto_button(AutoButton::Win, "Win"),
            create_auto_button(AutoButton::Skip, "Skip"),
        ],
    );

    param().commands.spawn(bundle).id()
}

fn create_auto_button(button: AutoButton, text: &str) -> impl Bundle + use<> {
    (
        GameButton::Auto(button),
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(40.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::horizontal(Val::Px(5.0)),
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
        BorderRadius::all(Val::Px(4.0)),
        BorderColor::all(Color::BLACK),
        BackgroundColor(BUTTON_INACTIVE),
        children![(
            Text::new(text),
            TextFont {
                // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            TextShadow::default(),
        )],
    )
}
