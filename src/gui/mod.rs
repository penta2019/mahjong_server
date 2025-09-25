mod control;
mod debug;
mod mahjong;
mod menu;
mod slider;
mod util;

use bevy::prelude::*;

use mahjong::{Rx, Tx};

pub fn run(tx: Tx, rx: Rx) {
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
