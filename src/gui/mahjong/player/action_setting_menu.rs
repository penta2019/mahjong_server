use bevy::prelude::*;

use crate::gui::mahjong::param;

const MENU_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

pub struct ActionSettingMenu {}

impl ActionSettingMenu {
    pub fn new() -> Self {
        let bundle = (Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(50.0),
            bottom: Val::Percent(0.0),
            display: Display::Flex,
            align_items: AlignItems::Center,
            ..default()
        },);

        param().commands.spawn(bundle);
        Self {}
    }
}

fn create_action_setting_button(text: &str) -> impl Bundle + use<> {
    (
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(2.0)),
            margin: UiRect::all(Val::Px(5.0)),
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BackgroundColor(MENU_BACKGROUND),
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
