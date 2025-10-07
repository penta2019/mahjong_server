use bevy::{
    gltf::GltfMaterialName,
    image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    input::mouse::MouseMotion,
    picking::pointer::PointerInteraction,
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    scene::SceneInstanceReady,
    shader::ShaderRef,
};

use super::super::move_animation::MoveAnimation;
use crate::model::Tile;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<TileMaterial>::default())
            .add_observer(tile_texture)
            .add_message::<HoveredTile>()
            // .add_systems(Startup, setup)
            .add_systems(Update, on_image_loaded)
            .add_systems(PreUpdate, tile_hover)
            .add_systems(PostUpdate, tile_update);
    }
}

#[derive(Message, Debug)]
pub struct HoveredTile {
    pub tile_entity: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct TileControl {
    tile: Tile,
    material: Option<Handle<TileMaterial>>,
    request_mutate: Option<Tile>,
    request_blend: Option<LinearRgba>,
}

impl TileControl {
    pub fn mutate(&mut self, tile: Tile) {
        self.request_mutate = Some(tile);
    }

    pub fn blend(&mut self, color: LinearRgba) {
        self.request_blend = Some(color);
    }
}

const SHADER_ASSET_PATH: &str = "tile/tile.wgsl";

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TileMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub blend: Vec4,
}

impl Material for TileMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

#[derive(Component, Debug)]
struct TileMeshParent {
    tile_entity: Entity,
}

pub fn create_tile(commands: &mut Commands, asset_server: &AssetServer, tile: Tile) -> Entity {
    let tile_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile/tile.glb"));
    commands
        .spawn((
            Name::new(format!("Tile({tile})")),
            SceneRoot(tile_model),
            TileControl {
                tile,
                material: None,
                request_mutate: None,
                request_blend: None,
            },
        ))
        .id()
}

fn tile_texture(
    ready: On<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_materials: ResMut<Assets<TileMaterial>>,
    childrens: Query<&Children>,
    mut tile_controls: Query<&mut TileControl>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    let e_tile = ready.event_target();
    let Ok(mut tile_control) = tile_controls.get_mut(e_tile) else {
        return;
    };

    // 牌のテクスチャを適切なものに張替え
    // ハイライト表示を行うために見た目が同じでも牌毎に異なるmaterialを作成
    for e_descendant in childrens.iter_descendants(e_tile) {
        if let Ok(name) = gltf_materials.get(e_descendant)
            && name.0.as_str() == "default"
        {
            let texture = asset_server.load(tile_path(tile_control.tile));
            let material = tile_materials.add(TileMaterial {
                texture,
                blend: Vec4::new(0.0, 0.0, 0.0, 0.0),
            });
            commands
                .entity(e_descendant)
                .remove::<MeshMaterial3d<StandardMaterial>>();
            commands.entity(e_descendant).insert((
                MeshMaterial3d(material.clone()),
                TileMeshParent {
                    tile_entity: e_tile,
                },
            ));
            tile_control.material = Some(material);
        }
    }
}

fn on_image_loaded(
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } = event
            && let Some(image) = images.get_mut(*id)
        {
            // TODO: Tile Texture以外にもこの設定が反映される
            let sampler = ImageSamplerDescriptor {
                mag_filter: ImageFilterMode::Linear,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                anisotropy_clamp: 8,
                ..default()
            };
            image.sampler = ImageSampler::Descriptor(sampler);
        }
    }
}

fn tile_hover(
    mut mouse_events: MessageReader<MouseMotion>,
    tile_move: Query<&TileControl, Changed<Transform>>,
    pointers: Query<&PointerInteraction>,
    tile_meshes: Query<&TileMeshParent>,
    move_animations: Query<(Entity, &mut MoveAnimation)>,
    mut tile_hover: MessageWriter<HoveredTile>,
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

    for (entity, _hit) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
    {
        // アニメーション中のものは除外
        if let Ok(m) = tile_meshes.get(*entity)
            && move_animations.get(m.tile_entity).is_err()
        {
            // println!("{entity} {_hit:?}");
            tile_hover.write(HoveredTile {
                tile_entity: Some(m.tile_entity),
            });
            return;
        }
    }

    tile_hover.write(HoveredTile { tile_entity: None });
}

fn tile_update(
    tile_controls: Query<&mut TileControl, Changed<TileControl>>,
    asset_server: Res<AssetServer>,
    mut tile_materials: ResMut<Assets<TileMaterial>>,
) {
    for mut tile_control in tile_controls {
        let Some(handle) = tile_control.material.as_ref() else {
            continue;
        };
        let material = tile_materials.get_mut(handle).unwrap();

        if let Some(tile) = tile_control.request_mutate.take() {
            let texture = asset_server.load(tile_path(tile));
            tile_control.tile = tile;
            material.texture = texture;
        }

        if let Some(color) = tile_control.request_blend.take() {
            material.blend = Vec4::new(color.red, color.green, color.blue, color.alpha);
        }
    }
}

fn tile_path(tile: Tile) -> String {
    format!("tile/texture_ktx2/{}.ktx2", tile)
}
