use bevy::{gltf::GltfMaterialName, prelude::*, scene::SceneInstanceReady};

use super::HasEntity;
use crate::model::Tile;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(amend_tile_texture)
            .add_event::<TileMutateEvent>()
            .add_systems(Update, tile_mutate_event)
            // StageのUpdateと同時実行するとremoveとinsertが重なって
            // insertしたばかりのMoveToが削除されることがあるのでPostUpdateを使用
            .add_systems(PostUpdate, animate_move);
    }
}

#[derive(Debug)]
pub struct GuiTile {
    entity: Entity,
    tile: Tile,
}

impl GuiTile {
    pub const WIDTH: f32 = 0.020;
    pub const HEIGHT: f32 = 0.028;
    pub const DEPTH: f32 = 0.016;

    pub fn new(commands: &mut Commands, asset_server: &AssetServer, tile: Tile) -> Self {
        let tile_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));
        let entity = commands
            .spawn((
                Name::new(format!("Tile({tile})")),
                SceneRoot(tile_model),
                TileTag {
                    tile,
                    mesh_materials: vec![],
                },
            ))
            .id();
        Self { entity, tile }
    }

    pub fn tile(&self) -> Tile {
        self.tile
    }
}

impl HasEntity for GuiTile {
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Component, Debug)]
pub struct TileTag {
    tile: Tile,
    mesh_materials: Vec<Handle<StandardMaterial>>,
}

impl TileTag {
    pub fn set_highlight(&self, materials: &mut Assets<StandardMaterial>, flag: bool) {
        for h in &self.mesh_materials {
            let material = materials.get_mut(h).unwrap();
            material.emissive = if flag {
                LinearRgba::new(0.1, 0.05, 0., 0.)
            } else {
                LinearRgba::BLACK
            };
        }
    }
}

#[derive(Component, Debug)]
pub struct TileMesh {
    tile_entity: Entity,
}

impl TileMesh {
    pub fn tile_entity(&self) -> Entity {
        self.tile_entity
    }
}

#[derive(Event, Debug)]
pub struct TileMutateEvent {
    entity: Entity,
    tile: Tile,
}

impl TileMutateEvent {
    pub fn mutate(tile: &mut GuiTile, m_tile: Tile) -> Self {
        tile.tile = m_tile;
        Self {
            entity: tile.entity,
            tile: m_tile,
        }
    }
}

fn amend_tile_texture(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    childrens: Query<&Children>,
    mut tile_tags: Query<&mut TileTag>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    let e_tile = trigger.target();
    let Ok(mut tile_tag) = tile_tags.get_mut(e_tile) else {
        return;
    };
    // テクスチャ張替え用のコンポーネントは以降不要なので削除
    // commands.entity(e_tile).remove::<TileTag>();

    // 牌のテクスチャを適切なものに張替え
    // ハイライト表示を行うために見た目が同じでも牌毎に異なるmaterialを作成
    for e_descendant in childrens.iter_descendants(e_tile) {
        if let Ok(name) = gltf_materials.get(e_descendant) {
            let material = materials.add(match name.0.as_str() {
                "face" => {
                    let texture = asset_server.load(format!("texture/{}.png", tile_tag.tile));
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
                TileMesh {
                    tile_entity: e_tile,
                },
            ));
            tile_tag.mesh_materials.push(material)
        }
    }
}

fn tile_mutate_event(
    mut reader: EventReader<TileMutateEvent>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut tile_tags: Query<&mut TileTag>,
) {
    for ev in reader.read() {
        let e_tile = ev.entity;
        let Ok(mut tile_tag) = tile_tags.get_mut(e_tile) else {
            return;
        };

        tile_tag.tile = ev.tile;
        for h in &tile_tag.mesh_materials {
            let material = materials.get_mut(h).unwrap();
            if material.base_color_texture.is_some() {
                let texture = asset_server.load(format!("texture/{}.png", tile_tag.tile));
                material.base_color_texture = Some(texture);
            }
        }
    }
}

// 等速移動アニメーション
#[derive(Component, Debug)]
pub struct MoveTo {
    // 移動アニメーションの目標(終了)位置
    target: Vec3,
    // アニメーションの残りフレーム数
    // フレームごとに値を1づつ下げていき, 1/frame_left * (target - 現在位置)つづ移動
    // frame_left == 1のときはtargetをそのまま現在位置にセットしてanimationを終了 (= MoveToを削除)
    frame_left: usize,
}

impl MoveTo {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            frame_left: 12,
        }
    }
}

fn animate_move(mut commands: Commands, move_tos: Query<(Entity, &mut Transform, &mut MoveTo)>) {
    for (e, mut tf, mut move_to) in move_tos {
        if move_to.frame_left > 1 {
            let diff_vec = move_to.target - tf.translation;
            tf.translation += 1.0 / move_to.frame_left as f32 * diff_vec;
            move_to.frame_left -= 1;
        } else {
            tf.translation = move_to.target;
            commands.entity(e).remove::<MoveTo>();
        }
    }
}
