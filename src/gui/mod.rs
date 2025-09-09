mod control;
mod debug;
mod menu;
mod slider;
mod stage;
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
            .add_plugins(stage::StagePlugin)
            .add_plugins(tile::TilePlugin);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(WireframePlugin::default());

        app.add_systems(Startup, setup);

        app.run();
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    use self::tile::*;
    use crate::model::{TM, TP, TS, Tile};

    for ti in [TM, TP, TS] {
        for ni in 0..10 {
            let tile = create_tile(&mut commands, &asset_server, Tile(ti, ni));
            commands.entity(tile).insert(Transform::from_xyz(
                TILE_WIDTH * ni as f32 - 0.1,
                TILE_HEIGHT / 2.,
                -0.1 * ti as f32,
            ));
        }
    }
}
