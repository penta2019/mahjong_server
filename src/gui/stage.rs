use bevy::{color::palettes::basic::GREEN, prelude::*, scene::SceneInstanceReady};

use super::tile::*;
use crate::model::*;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_observer(on_instantiated);
    }
}

#[derive(Component, Debug)]
pub struct GuiStage;

#[derive(Component, Debug)]
pub struct GuiPlayer {
    seat: Seat,
}

#[derive(Component, Debug)]
pub struct GuiHand;

#[derive(Component, Debug)]
pub struct GuiMeld;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    scenes: Res<Assets<Scene>>,
) {
    // Light
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

    // Stage
    let stage = commands
        .spawn((
            Transform::from_xyz(0., 0., 0.),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
            MeshMaterial3d(materials.add(Color::from(GREEN))),
            GuiStage,
        ))
        .id();

    for s in 0..SEAT {
        let player = commands
            .spawn((
                ChildOf(stage),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * s as f32),
                    ..Default::default()
                },
                GuiPlayer { seat: s },
            ))
            .id();

        let hand = commands
            .spawn((
                ChildOf(player),
                Transform::from_xyz(-0.12, 0., 0.20),
                GuiHand,
            ))
            .id();

        for i in 0..13 {
            let tile = super::tile::create_tile(&mut commands, &asset_server, Tile(TM, 5));
            commands.entity(tile).insert((
                ChildOf(hand),
                Transform::from_xyz(TILE_WIDTH * i as f32, TILE_HEIGHT / 2., 0.01 as f32),
            ));
        }
    }

    // let stage_model = asset_server.load("stage_layout.glb#Scene0");
    // commands.spawn((SceneRoot(stage_model), GuiStage));
}

fn on_instantiated(
    trigger: Trigger<SceneInstanceReady>,
    gui_stage: Query<Entity, With<GuiStage>>,
    query_names: Query<&Name>,
    query_children: Query<&Children>,
) {
    let Ok(entity) = gui_stage.get(trigger.target()) else {
        return;
    };

    super::util::print_hierarchy(entity, &query_names, &query_children);
}
