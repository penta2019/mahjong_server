use bevy::{gltf::GltfMaterialName, prelude::*, scene::SceneInstanceReady};

use super::StageParam;
use crate::model::Tile;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(amend_tile_texture)
            .add_systems(Update, animate_move);
    }
}

#[derive(Debug)]
pub struct GuiTile {
    pub entity: Entity,
    pub tile: Tile,
}

impl GuiTile {
    pub const WIDTH: f32 = 0.020;
    pub const HEIGHT: f32 = 0.028;
    pub const DEPTH: f32 = 0.016;

    pub fn new(param: &mut StageParam, tile: Tile) -> Self {
        let tile_model = param
            .asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));
        let entity = param
            .commands
            .spawn((
                Name::new(format!("Tile({tile})")),
                SceneRoot(tile_model),
                IntoTile { tile },
            ))
            .id();
        Self { entity, tile }
    }
}

#[derive(Component, Debug)]
struct IntoTile {
    tile: Tile,
}

fn amend_tile_texture(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    q_children: Query<&Children>,
    q_into_tile: Query<&IntoTile>,
    q_mesh_materials: Query<&GltfMaterialName>,
) {
    let e_tile = trigger.target();
    let Ok(gui_tile) = q_into_tile.get(e_tile) else {
        return;
    };
    // テクスチャ張替え用のコンポーネントは以降不要なので削除
    commands.entity(e_tile).remove::<IntoTile>();

    // 牌のテクスチャを適切なものに張替え
    for e_descendant in q_children.iter_descendants(trigger.target()) {
        if let Ok(name) = q_mesh_materials.get(e_descendant) {
            if name.0 != "face" {
                continue;
            }
            let texture = asset_server.load(format!("texture/{}.png", gui_tile.tile));
            let material = asset_materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..Default::default()
            });
            commands
                .entity(e_descendant)
                .insert(MeshMaterial3d(material));
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

fn animate_move(mut commands: Commands, q_move: Query<(Entity, &mut Transform, &mut MoveTo)>) {
    for (e, mut tf, mut move_to) in q_move {
        let diff_vec = move_to.target - tf.translation;
        tf.translation += 1.0 / move_to.frame_left as f32 * diff_vec;
        move_to.frame_left -= 1;
        if move_to.frame_left == 0 {
            commands.entity(e).remove::<MoveTo>();
        }
    }
}
