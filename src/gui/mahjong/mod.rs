mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod tile;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::util::reparent_tranform;
use crate::model::*;

pub use stage::StagePlugin;
pub use tile::TilePlugin;

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use player::{GuiPlayer, HandMode};
use stage::{create_tile, param};
use tile::{GuiTile, MoveTo, TileMutateEvent, TileTag};

trait HasEntity {
    fn entity(&self) -> Entity;
}
