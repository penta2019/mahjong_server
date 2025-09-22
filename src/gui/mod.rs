mod control;
mod debug;
mod mahjong;
mod menu;
mod slider;
mod util;

use std::sync::mpsc::{Receiver, Sender};

use bevy::prelude::*;

use crate::model::{ClientMessage, ServerMessage};

pub fn run(tx: Sender<ClientMessage>, rx: Receiver<ServerMessage>) {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(slider::SliderPlugin)
        .add_plugins(control::ControlPlugin)
        .add_plugins(debug::DebugPlugin)
        .add_plugins(menu::MenuPlugin)
        .add_plugins(mahjong::StagePlugin::new(tx, rx))
        .add_plugins(mahjong::TilePlugin);
    app.run();
}
