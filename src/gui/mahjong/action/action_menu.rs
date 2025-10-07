use bevy::prelude::*;

use super::{BUTTON_INACTIVE, GameButton};
use crate::{
    gui::mahjong::param,
    model::{Action, ActionType},
};

// use super::tile::GuiTile;

pub fn create_main_action_menu(action_types: &[ActionType]) -> Entity {
    let param = param();
    let menu = param
        .commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    for ty in action_types {
        param
            .commands
            .spawn(create_main_action_button(*ty, &format!("{:?}", *ty)))
            .insert(ChildOf(menu));
    }

    menu
}

pub fn create_sub_action_menu(actions: &[Action]) -> Entity {
    let param = param();
    let menu = param
        .commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    param
        .commands
        .spawn(create_main_action_button(ActionType::Nop, "Cancel"))
        .insert(ChildOf(menu));

    for act in actions {
        param
            .commands
            .spawn(create_sub_action_button(act.clone()))
            .insert(ChildOf(menu));
    }

    menu
}

fn create_main_action_button(ty: ActionType, text: &str) -> impl Bundle + use<> {
    (
        GameButton::Main(ty),
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(40.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::all(Val::Px(5.0)),
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
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

fn create_sub_action_button(action: Action) -> impl Bundle + use<> {
    // let tiles: Vec<_> = action.tiles.iter().map(|t| GuiTile::new(*t)).collect();
    let text: String = action.tiles.iter().map(|t| t.to_string()).collect();
    (
        GameButton::Sub(action),
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::all(Val::Px(5.0)),
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
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
