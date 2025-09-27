mod control;
mod param;
mod stage_plugin;
mod tile_plugin;

use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::prelude::*;

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
        app.add_plugins((
            tile_plugin::TilePlugin,
            stage_plugin::StagePlugin::new(tx, rx),
        ));
    }
}
