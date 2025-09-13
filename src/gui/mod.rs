mod control;
mod debug;
mod mahjong;
mod menu;
mod slider;
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
            .add_plugins(mahjong::StagePlugin::new(event_rx))
            .add_plugins(mahjong::TilePlugin);

        app.run();
    }
}
