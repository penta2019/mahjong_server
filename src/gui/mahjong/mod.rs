mod button;
mod control;
mod control_param;
mod tile_plugin;

use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;

use self::{
    control::MahjonGuiControl,
    control_param::{ControlParam, with_param},
};
use crate::model::{ClientMessage, ServerMessage};

pub type Tx = Sender<ClientMessage>;
pub type Rx = Receiver<ServerMessage>;

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
        app.add_plugins(tile_plugin::TilePlugin)
            .insert_resource(MahjonGuiControl::new(tx, rx))
            .add_systems(Update, control_loop);
    }
}

fn control_loop(mut param: ControlParam, mut stage_res: ResMut<MahjonGuiControl>) {
    with_param(&mut param, || {
        stage_res.update();
    });
}
