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

pub use self::stage::GuiStage;

use self::{
    discard::GuiDiscard,
    hand::GuiHand,
    meld::GuiMeld,
    player::{GuiPlayer, HandMode},
    tile::GuiTile,
};

trait HasEntity {
    fn entity(&self) -> Entity;
}
