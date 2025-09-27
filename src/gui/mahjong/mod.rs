mod discard;
mod hand;
mod meld;
mod move_to;
mod player;
mod stage;
mod stage_param;
mod stage_plugin;
mod tile;

use std::{
    f32::consts::{FRAC_PI_2, PI},
    sync::Mutex,
};

use bevy::prelude::*;

use super::util::reparent_tranform;
use crate::model::*;

pub use stage_plugin::{Rx, Tx};

use discard::GuiDiscard;
use hand::GuiHand;
use meld::GuiMeld;
use move_to::MoveTo;
use player::{GuiPlayer, HandMode, PossibleActions};
use stage::GuiStage;
use stage_param::{StageParam, create_tile, param, with_param};
use tile::{GuiTile, HoveredTile, TileTag};

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
            move_to::MovePlugin,
            tile::TilePlugin,
            stage_plugin::StagePlugin::new(tx, rx),
        ));
    }
}
