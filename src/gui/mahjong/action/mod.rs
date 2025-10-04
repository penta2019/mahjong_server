mod action_control;
mod action_menu;
mod auto_menu;

use bevy::prelude::*;

use crate::model::{Action, ActionType};

pub use self::{action_control::ActionControl, auto_menu::AutoButton};

const BUTTON_BACKGROUND: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

#[derive(Component, Debug, PartialEq, Eq)]
pub enum GameButton {
    Main(ActionType),
    Sub(Action),
    Auto(AutoButton),
}
