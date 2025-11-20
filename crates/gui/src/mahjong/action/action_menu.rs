use super::{super::prelude::*, BUTTON_INACTIVE, GameButton};
use crate::ui3d::Ui3dTransform;

pub fn create_main_action_menu(action_types: &[ActionType]) -> Entity {
    let cmd = cmd();
    let menu = cmd
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();

    for ty in action_types {
        cmd.spawn(create_main_action_button(*ty, &format!("{:?}", *ty)))
            .insert(ChildOf(menu));
    }

    menu
}

pub fn create_sub_action_menu(actions: &[Action]) -> (Entity, Vec<Entity>) {
    let cmd = cmd();
    let menu = cmd
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Percent(20.0),
            bottom: Val::Percent(18.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::End,
            align_items: AlignItems::End,
            ..default()
        })
        .id();

    let mut tile_sets = vec![];
    for act in actions {
        let button = cmd
            .spawn(create_sub_action_button(act.clone()))
            .insert(ChildOf(menu))
            .id();
        if !act.tiles.is_empty() {
            let tile_set = create_tile_set(&act.tiles);
            cmd.entity(tile_set)
                .insert(Ui3dTransform::new(button, Quat::IDENTITY, Vec3::ONE));
            tile_sets.push(tile_set);
        }
    }

    cmd.spawn(create_main_action_button(ActionType::Nop, "Cancel"))
        .insert(ChildOf(menu));

    (menu, tile_sets)
}

fn create_main_action_button(ty: ActionType, text: &str) -> impl Bundle {
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
        children![create_text(text.into(), 16.0)],
    )
}

fn create_sub_action_button(action: Action) -> impl Bundle {
    // let text = create_text(action.tiles.iter().map(|t| t.to_string()).collect(), 16.0);
    (
        GameButton::Sub(action),
        Button,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(1.0)),
            margin: UiRect::all(Val::Px(5.0)),
            flex_direction: FlexDirection::Column,
            // 内部のテキストを中央に表示(横方向)
            justify_content: JustifyContent::Center,
            // 内部のテキストを中央に表示(縦方向)
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::BLACK),
        BackgroundColor(BUTTON_INACTIVE),
        // children![text],
    )
}

pub fn create_tile_set(m_tiles: &Vec<Tile>) -> Entity {
    let entity = cmd().spawn(Name::new("TileSet")).id();

    // entityの座標が中心になるように配置
    let mut x = -GuiTile::WIDTH / 2.0 * (m_tiles.len() - 1) as f32;
    for m_tile in m_tiles {
        let tile = GuiTile::ui(*m_tile);
        tile.insert((ChildOf(entity), Transform::from_xyz(x, 0.0, 0.0)));
        x += GuiTile::WIDTH;
    }
    entity
}
