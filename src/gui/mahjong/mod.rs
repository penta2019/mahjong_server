mod discard;
mod hand;
mod meld;
mod param;
mod player;
mod stage;
mod stage_plugin;
mod tile;
mod tile_plugin;

use std::{
    f32::consts::{FRAC_PI_2, PI},
    sync::Mutex,
};

use bevy::prelude::*;

use super::{move_animation::MoveAnimation, util::reparent_tranform};
use crate::model::*;

pub use stage_plugin::{Rx, Tx};

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use param::{StageParam, param, with_param};
use player::{GuiPlayer, HandMode, PossibleActions};
use stage::GuiStage;
use tile::GuiTile;
use tile_plugin::{HoveredTile, TileTag, create_tile};

trait HasEntity {
    fn entity(&self) -> Entity;
}

pub struct MahjongPlugin {
    txrx: Mutex<Option<(Tx, Rx)>>,
}

impl MahjongPlugin {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            txrx: Mutex::new(Some((tx, rx))),
        }
    }
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = self.txrx.lock().unwrap().take().unwrap();
        app.add_plugins((
            tile_plugin::TilePlugin,
            stage_plugin::StagePlugin::new(tx, rx),
        ));
    }
}
