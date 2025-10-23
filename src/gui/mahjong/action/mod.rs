mod action_control;
mod action_menu;
mod auto_menu;

use bevy::{ecs::system::SystemParam, input::mouse::MouseButtonInput};

use super::{prelude::*, tile_plugin::HoveredTile};
use crate::model::{Action, ActionType};

pub use self::{action_control::ActionControl, auto_menu::AutoButton};

const BUTTON_ACTIVE: Color = Color::srgba(0.15, 0.40, 0.15, 0.8);
const BUTTON_INACTIVE: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

#[derive(SystemParam)]
pub struct ActionParam<'w, 's> {
    // Game Button
    game_buttons: Query<
        'w,
        's,
        (
            &'static mut GameButton,
            &'static mut BorderColor,
            &'static mut BackgroundColor,
        ),
    >, // 実装側から参照できるようにInteractionから分離
    game_button_interactions:
        Query<'w, 's, (Entity, &'static Interaction), (Changed<Interaction>, With<GameButton>)>,

    // MessageReader
    hovered_tile: MessageReader<'w, 's, HoveredTile>,
    mouse_input: MessageReader<'w, 's, MouseButtonInput>,
}

#[derive(Component, Debug)]
pub enum GameButton {
    Main(ActionType),
    Sub(Action),
    Auto(AutoButton),
}
