mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod stage_param;
mod stage_plugin;
mod tile;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use super::util::reparent_tranform;
use crate::model::*;

pub use stage_plugin::{Rx, StagePlugin, Tx};
pub use tile::TilePlugin;

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use player::{GuiPlayer, HandMode, PossibleActions};
use stage::GuiStage;
use stage_param::{StageParam, create_tile, param, with_param};
use tile::{GuiTile, HoveredTile, MoveTo, TileMutate, TileTag};

trait HasEntity {
    fn entity(&self) -> Entity;
}
