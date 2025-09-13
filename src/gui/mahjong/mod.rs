mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod tile;

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::model::*;

pub use stage::StagePlugin;
pub use tile::TilePlugin;

use discard::GuiDiscard;
use hand::GuiHand;
use meld::{GuiMeld, GuiMeldItem};
use player::GuiPlayer;
use stage::StageParam;
use tile::{GuiTile, MoveTo};

trait HasEntity {
    fn entity(&self) -> Entity;
}
