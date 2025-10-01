use bevy::prelude::*;

use crate::model::{Action, ActionType};

// use super::tile::GuiTile;

const MENU_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

#[derive(Component, Debug, PartialEq, Eq)]
pub enum ActionButton {
    Main(ActionType),
    Sub(Action),
}

pub fn create_action_main_button(ty: ActionType, text: &str) -> impl Bundle + use<> {
    (
        ActionButton::Main(ty),
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(40.0),
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

pub fn create_action_sub_button(action: Action) -> impl Bundle + use<> {
    // let tiles: Vec<_> = action.tiles.iter().map(|t| GuiTile::new(*t)).collect();
    let text: String = action.tiles.iter().map(|t| t.to_string()).collect();
    (
        ActionButton::Sub(action),
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
