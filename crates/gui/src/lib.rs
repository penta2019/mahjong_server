#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod camera;
mod debug;
mod mahjong;
mod menu;
mod move_animation;
mod slider;
mod util;

use bevy::prelude::*;

use self::{
    camera::{CameraState, ViewportMode},
    debug::DebugState,
    mahjong::{Rx, Tx},
    menu::MenuState,
};

#[derive(Resource, Debug)]
struct Context {
    can_fly: bool,
}

pub fn run(tx: Tx, rx: Rx) {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        MeshPickingPlugin,
        camera::CameraPlugin::new(ViewportMode::AspectRatio(4.0 / 3.0)),
        debug::DebugPlugin,
        slider::SliderPlugin,
        menu::MenuPlugin,
        move_animation::MoveAnimationPlugin,
        mahjong::MahjongPlugin::new(tx, rx),
    ))
    .insert_resource(Context { can_fly: false })
    .add_systems(Update, keyboard_handler);
    app.run();
}

fn keyboard_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut context: ResMut<Context>,
    mut next_camera_state: ResMut<NextState<CameraState>>,
    debug_state: Res<State<DebugState>>,
    mut next_debug_state: ResMut<NextState<DebugState>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        next_debug_state.set(if **debug_state == DebugState::Visible {
            DebugState::Hidden
        } else {
            DebugState::Visible
        });
    }

    if keys.just_pressed(KeyCode::Space) {
        context.can_fly = !context.can_fly;
        if context.can_fly && **menu_state == MenuState::Hidden {
            next_camera_state.set(CameraState::Fly);
        } else {
            next_camera_state.set(CameraState::Fix);
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        match **menu_state {
            MenuState::Visible => {
                // メニューを閉じる State: Disabled -> On, Off -> Off
                next_menu_state.set(MenuState::Hidden);
                if context.can_fly {
                    next_camera_state.set(CameraState::Fly);
                }
            }
            MenuState::Hidden => {
                // メニューを開く State: On -> Disabled, Off -> Off
                next_menu_state.set(MenuState::Visible);
                next_camera_state.set(CameraState::Fix);
            }
        }
    }
}
