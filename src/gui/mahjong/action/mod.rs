mod action_control;
mod action_menu;
mod auto_menu;

use bevy::prelude::{Color, Component};

use crate::model::{Action, ActionType};

pub use self::{action_control::ActionControl, auto_menu::AutoButton};

pub const BUTTON_ACTIVE: Color = Color::srgba(0.15, 0.40, 0.15, 0.8);
pub const BUTTON_INACTIVE: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

#[derive(Component, Debug)]
pub enum GameButton {
    Main(ActionType),
    Sub(Action),
    Auto(AutoButton),
}
