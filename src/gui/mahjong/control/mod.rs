mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod tile;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::param::param;

use crate::{
    gui::{move_animation::MoveAnimation, util::reparent_tranform},
    model::*,
};

pub use player::PossibleActions;
pub use stage::GuiStage;

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use player::{GuiPlayer, HandMode};
use tile::GuiTile;

trait HasEntity {
    fn entity(&self) -> Entity;
}
