use bevy::{gltf::GltfMaterialName, prelude::*, scene::SceneInstanceReady};

use crate::model::Tile;

pub const TILE_WIDTH: f32 = 0.020;
pub const TILE_HEIGHT: f32 = 0.028;
pub const TILE_DEPTH: f32 = 0.016;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(amend_tile_texture);
    }
}

#[derive(Component, Debug)]
struct GuiTile {
    tile: Tile,
}

pub fn create_tile(commands: &mut Commands, asset_server: &AssetServer, tile: Tile) -> Entity {
    let tile_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));
    commands
        .spawn((SceneRoot(tile_model), GuiTile { tile }))
        .id()
}

fn amend_tile_texture(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    children: Query<&Children>,
    gui_tile: Query<&GuiTile>,
    mesh_materials: Query<&GltfMaterialName>,
) {
    let Ok(gui_tile) = gui_tile.get(trigger.target()) else {
        return;
    };

    // 牌のテクスチャを適切なものに張替え
    for descendant in children.iter_descendants(trigger.target()) {
        if let Ok(name) = mesh_materials.get(descendant) {
            if name.0 != "face" {
                continue;
            }
            let texture = asset_server.load(format!("texture/{}.png", gui_tile.tile));
            let material = asset_materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..Default::default()
            });
            commands.entity(descendant).insert(MeshMaterial3d(material));
        }
    }
}
