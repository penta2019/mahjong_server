mod discard;
mod hand;
mod meld;
mod player;
mod plugin;
mod stage;
mod tile;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::util::reparent_tranform;
use crate::model::*;

pub use plugin::{Rx, StagePlugin, Tx};
pub use tile::TilePlugin;

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use player::{GuiPlayer, HandMode};
use plugin::{create_tile, param};
use stage::GuiStage;
use tile::{GuiTile, MoveTo, TileMutateEvent, TileTag};

trait HasEntity {
    fn entity(&self) -> Entity;
}
