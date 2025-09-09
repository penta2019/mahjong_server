use bevy::{
    color::palettes::basic::GREEN, gltf::GltfMaterialName, prelude::*, scene::SceneInstanceReady,
};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_observer(change_tile_texture);
    }
}

#[derive(Component)]
struct TileOverride(String);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ライト
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // 床
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0).subdivisions(10))),
        MeshMaterial3d(materials.add(Color::from(GREEN))),
        Transform::from_xyz(0.0, -0.014, 0.0),
    ));

    for (ti, t) in ["m", "p", "s"].iter().enumerate() {
        for n in 0..10 {
            let tile = asset_server.load(GltfAssetLabel::Scene(0).from_asset("tile.glb"));
            commands.spawn((
                SceneRoot(tile.clone()),
                TileOverride(format!("{t}{n}")),
                Transform::from_xyz(0.02 * n as f32 - 0.1, 0.0, -0.1 * ti as f32),
            ));
        }
    }
}

fn change_tile_texture(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
    children: Query<&Children>,
    tile_override: Query<&TileOverride>,
    mesh_materials: Query<&GltfMaterialName>,
) {
    let Ok(tile_override) = tile_override.get(trigger.target()) else {
        return;
    };

    for descendant in children.iter_descendants(trigger.target()) {
        if let Ok(name) = mesh_materials.get(descendant) {
            if name.0 != "face" {
                continue;
            }
            let texture = asset_server.load(format!("texture/{}.png", tile_override.0));
            let material = asset_materials.add(StandardMaterial {
                base_color_texture: Some(texture),
                ..Default::default()
            });
            commands.entity(descendant).insert(MeshMaterial3d(material));
        }
    }
}
