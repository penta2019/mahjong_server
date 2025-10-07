mod action;
mod control;
mod control_param;
mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod tile;
mod tile_plugin;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use self::{
    action::ActionControl,
    control::MahjonGuiControl,
    control_param::{ControlParam, param, with_param},
    discard::GuiDiscard,
    hand::{GuiHand, IsDrawn},
    meld::GuiMeld,
    player::{GuiPlayer, HandMode},
    stage::GuiStage,
    tile::{GuiTile, TILE_ACTIVE, TILE_INACTIVE, TILE_NORMAL},
};
use crate::model::{Event as MjEvent, *};

pub type Tx = std::sync::mpsc::Sender<ClientMessage>;
pub type Rx = std::sync::mpsc::Receiver<ServerMessage>;

trait HasEntity {
    fn entity(&self) -> Entity;
}

pub struct MahjongPlugin {
    txrx: std::sync::Mutex<Option<(Tx, Rx)>>,
}

impl MahjongPlugin {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            txrx: std::sync::Mutex::new(Some((tx, rx))),
        }
    }
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = self.txrx.lock().unwrap().take().unwrap();
        app.add_plugins(tile_plugin::TilePlugin)
            .insert_resource(MahjonGuiControl::new(tx, rx))
            .add_systems(Update, control_loop);
    }
}

fn control_loop(mut param: ControlParam, mut stage_control: ResMut<MahjonGuiControl>) {
    with_param(&mut param, || {
        stage_control.update();
    });
}
