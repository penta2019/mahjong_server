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
            .add_systems(PostUpdate, (tile_mutate, tile_blend));
    }
}

#[derive(Message, Debug)]
pub struct HoveredTile {
    pub entity: Option<Entity>,
}

#[derive(Component, Debug)]
pub struct TileMutate(pub Tile);

#[derive(Component, Debug)]
pub struct TileBlend(pub LinearRgba);

#[derive(Component, Debug)]
struct TileInit(Tile);

#[derive(Component, Debug)]
struct TileComponent {
    material: Handle<TileMaterial>,
}

#[derive(Component, Debug)]
struct TileMesh {
    entity: Entity,
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

pub fn create_tile(commands: &mut Commands, asset_server: &AssetServer, tile: Tile) -> Entity {
    let tile_model = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile/tile.glb"));
    commands
        .spawn((
            Name::new(format!("Tile({tile})")),
            SceneRoot(tile_model),
            TileInit(tile),
        ))
        .id()
}

fn tile_texture(
    ready: On<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_materials: ResMut<Assets<TileMaterial>>,
    tile_inits: Query<&TileInit>,
    childrens: Query<&Children>,
    gltf_materials: Query<&GltfMaterialName>,
) {
    let e_tile = ready.event_target();
    let Ok(TileInit(tile)) = tile_inits.get(e_tile) else {
        return; // Tile以外の物も流れてくることに注意
    };

    // 牌のテクスチャを適切なものに張替え
    // ハイライト表示を行うために見た目が同じでも牌毎に異なるmaterialを作成
    for e_descendant in childrens.iter_descendants(e_tile) {
        if let Ok(name) = gltf_materials.get(e_descendant)
            && name.0.as_str() == "default"
        {
            let texture = asset_server.load(tile_path(*tile));
            let material = tile_materials.add(TileMaterial {
                texture,
                blend: Vec4::new(0.0, 0.0, 0.0, 0.0),
            });
            commands
                .entity(e_descendant)
                .remove::<MeshMaterial3d<StandardMaterial>>()
                .insert((
                    MeshMaterial3d(material.clone()),
                    TileMesh { entity: e_tile },
                ));
            commands
                .entity(e_tile)
                .remove::<TileInit>()
                .insert(TileComponent { material });
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
    tile_move: Query<&TileComponent, Changed<Transform>>,
    pointers: Query<&PointerInteraction>,
    tile_meshes: Query<&TileMesh>,
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
            && move_animations.get(m.entity).is_err()
        {
            // println!("{entity} {_hit:?}");
            tile_hover.write(HoveredTile {
                entity: Some(m.entity),
            });
            return;
        }
    }

    tile_hover.write(HoveredTile { entity: None });
}

fn tile_mutate(
    mut commands: Commands,
    tiles: Query<(Entity, &TileComponent, &TileMutate)>,
    asset_server: Res<AssetServer>,
    mut tile_materials: ResMut<Assets<TileMaterial>>,
) {
    for (e_tile, component, TileMutate(tile)) in tiles {
        let material = tile_materials.get_mut(&component.material).unwrap();
        material.texture = asset_server.load(tile_path(*tile));
        commands.entity(e_tile).remove::<TileMutate>();
    }
}

fn tile_blend(
    mut commands: Commands,
    tiles: Query<(Entity, &TileComponent, &TileBlend)>,
    mut tile_materials: ResMut<Assets<TileMaterial>>,
) {
    for (e_tile, component, TileBlend(color)) in tiles {
        let material = tile_materials.get_mut(&component.material).unwrap();
        material.blend = Vec4::new(color.red, color.green, color.blue, color.alpha);
        commands.entity(e_tile).remove::<TileBlend>();
    }
}

fn tile_path(tile: Tile) -> String {
    format!("tile/texture_ktx2/{}.ktx2", tile)
}
