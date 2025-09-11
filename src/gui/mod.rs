mod control;
mod debug;
mod menu;
mod player;
mod slider;
mod stage;
mod tile;
mod util;

use bevy::prelude::*;

pub struct Gui {}

impl Gui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, event_rx: crate::listener::EventRx) {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .add_plugins(slider::SliderPlugin)
            .add_plugins(control::ControlPlugin)
            .add_plugins(debug::DebugPlugin)
            .add_plugins(menu::MenuPlugin)
            .add_plugins(stage::StagePlugin::new(event_rx))
            .add_plugins(tile::TilePlugin);

        app.run();
    }
}
