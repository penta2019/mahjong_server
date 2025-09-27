use bevy::{
    gltf::GltfMaterialName, input::mouse::MouseMotion, prelude::*, scene::SceneInstanceReady,
};

use super::super::control::MainCamera;
use crate::model::Tile;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(tile_texture)
            .add_event::<HoveredTile>()
            .add_systems(PreUpdate, tile_hover)
            .add_systems(PostUpdate, tile_mutate);
    }
}

#[derive(Event, Debug)]
pub struct HoveredTile {
    pub tile_entity: Option<Entity>,
}

pub fn create_tile(commands: &mut Commands, asset_server: &AssetServer, tile: Tile) -> Entity {
    let tile_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));
    commands
        .spawn((
            Name::new(format!("Tile({tile})")),
            SceneRoot(tile_model),
            TileControl {
                tile,
                mesh_materials: vec![],
            },
        ))
        .id()
}

#[derive(Component, Debug)]
pub struct TileControl {
    tile: Tile,
    mesh_materials: Vec<Handle<StandardMaterial>>,
}

impl TileControl {
    pub fn mutate(&mut self, tile: Tile) {
        self.tile = tile;
    }

    pub fn set_emissive(&self, materials: &mut Assets<StandardMaterial>, color: LinearRgba) {
        for h in &self.mesh_materials {
            let material = materials.get_mut(h).unwrap();
            material.emissive = color;
        }
    }
}

#[derive(Component, Debug)]
struct TileMeshParent {
    tile_entity: Entity,
}

fn tile_texture(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    childrens: Query<&Children>,
    mut tile_controls: Query<&mut TileControl>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    let e_tile = trigger.target();
    let Ok(mut tile_control) = tile_controls.get_mut(e_tile) else {
        return;
    };
    // テクスチャ張替え用のコンポーネントは以降不要なので削除
    // commands.entity(e_tile).remove::<TileControl>();

    // 牌のテクスチャを適切なものに張替え
    // ハイライト表示を行うために見た目が同じでも牌毎に異なるmaterialを作成
    for e_descendant in childrens.iter_descendants(e_tile) {
        if let Ok(name) = gltf_materials.get(e_descendant) {
            let material = materials.add(match name.0.as_str() {
                "face" => {
                    let texture = asset_server.load(format!("texture/{}.png", tile_control.tile));
                    StandardMaterial {
                        base_color_texture: Some(texture),
                        ..Default::default()
                    }
                }
                "base" => StandardMaterial {
                    base_color: Color::srgb_u8(0xda, 0xd9, 0xd9),
                    ..Default::default()
                },
                "back" => StandardMaterial {
                    base_color: Color::srgb_u8(0x00, 0x00, 0x00),
                    ..Default::default()
                },
                _ => continue,
            });
            commands.entity(e_descendant).insert((
                MeshMaterial3d(material.clone()),
                TileMeshParent {
                    tile_entity: e_tile,
                },
            ));
            tile_control.mesh_materials.push(material)
        }
    }
}

fn tile_mutate(
    tile_controls: Query<&mut TileControl, Changed<TileControl>>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for tile_control in tile_controls {
        println!("changed: {}", tile_control.tile);
        for h in &tile_control.mesh_materials {
            let material = materials.get_mut(h).unwrap();
            if material.base_color_texture.is_some() {
                let texture = asset_server.load(format!("texture/{}.png", tile_control.tile));
                material.base_color_texture = Some(texture);
            }
        }
    }
}

fn tile_hover(
    mut mouse_events: EventReader<MouseMotion>,
    tile_move: Query<&TileControl, Changed<Transform>>,
    window: Single<&mut Window>,
    camera: Single<(&mut Camera, &GlobalTransform), With<MainCamera>>,
    mut ray_cast: MeshRayCast,
    tile_meshes: Query<&TileMeshParent>,
    mut tile_hover: EventWriter<HoveredTile>,
) {
    // マウスか牌が移動した場合にのみ新しく判定を実行
    {
        let mut skip = true;
        for _ in mouse_events.read() {
            skip = false;
        }
        for _ in tile_move {
            skip = false;
        }
        if skip {
            return;
        }
    }

    let Some(p_cursor) = window.cursor_position() else {
        return;
    };
    let (camera, tf_camera) = &*camera;
    let Ok(ray) = camera.viewport_to_world(tf_camera, p_cursor) else {
        return;
    };

    for (entity, _hit) in ray_cast.cast_ray(ray, &MeshRayCastSettings::default()) {
        if let Ok(m) = tile_meshes.get(*entity) {
            tile_hover.write(HoveredTile {
                tile_entity: Some(m.tile_entity),
            });
            return;
        }
    }
    tile_hover.write(HoveredTile { tile_entity: None });
}
