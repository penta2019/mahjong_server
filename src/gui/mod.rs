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
    app.add_plugins((
        DefaultPlugins,
        slider::SliderPlugin,
        control::ControlPlugin,
        debug::DebugPlugin,
        menu::MenuPlugin,
        mahjong::MahjongPlugin::new(tx, rx),
    ));
    app.run();
}
