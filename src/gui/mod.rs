mod control;
mod debug;
mod menu;
mod slider;
mod tile;
mod util;

#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;

pub struct Gui {}

impl Gui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let mut app = App::new();
        app.add_plugins(DefaultPlugins)
            .add_plugins(slider::SliderPlugin)
            .add_plugins(control::ControlPlugin)
            .add_plugins(debug::DebugPlugin)
            .add_plugins(menu::MenuPlugin)
            .add_plugins(tile::TilePlugin);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(WireframePlugin::default());
        app.run();
    }
}
