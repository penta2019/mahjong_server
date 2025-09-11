use std::sync::Mutex;

use bevy::{
    color::palettes::basic::{BLACK, GREEN},
    ecs::system::SystemParam,
    prelude::*,
    // scene::SceneInstanceReady,
};

use super::tile::*;
use crate::{
    listener::EventRx,
    model::{self, SEAT, Seat},
};

#[derive(Resource, Debug)]
struct EventReceiver {
    recv: Mutex<EventRx>,
}

pub struct StagePlugin {
    event_rx: Mutex<Option<EventRx>>,
}

impl StagePlugin {
    pub fn new(event_rx: EventRx) -> Self {
        Self {
            event_rx: Mutex::new(Some(event_rx)),
        }
    }
}

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        let event_rx = self.event_rx.lock().unwrap().take().unwrap();
        app.insert_resource(EventReceiver {
            recv: Mutex::new(event_rx),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, read_event);
    }
}

#[derive(Component, Debug)]
pub struct GuiStage;

#[derive(Component, Debug)]
pub struct GuiPlayer {
    seat: Seat,
}

#[derive(Component, Debug)]
pub struct GuiHand {
    seat: Seat,
}

#[derive(Component, Debug)]
pub struct GuiMelds {
    seat: Seat,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // asset_server: Res<AssetServer>,
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

    // stage
    let stage = commands
        .spawn((
            Transform::from_xyz(0., 0., 0.),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
            MeshMaterial3d(materials.add(Color::from(GREEN))),
            GuiStage,
        ))
        .id();

    // info
    commands.spawn((
        ChildOf(stage),
        Transform::from_xyz(0., 0.001, 0.),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
        MeshMaterial3d(materials.add(Color::from(BLACK))),
    ));

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
                Transform::from_xyz(-0.12, 0., 0.21),
                GuiHand { seat: s },
            ))
            .id();

        // for i in 0..13 {
        //     let tile = super::tile::create_tile(&mut commands, &asset_server, Tile(model::TM, 5));
        //     commands.entity(tile).insert((
        //         ChildOf(hand),
        //         Transform::from_xyz(TILE_WIDTH * i as f32, TILE_HEIGHT / 2., 0.01 as f32),
        //     ));
        // }
    }
}

fn read_event(stg: StageQueries, event_reader: ResMut<EventReceiver>) {
    if let Ok(e) = event_reader.recv.lock().unwrap().try_recv() {
        println!("gui received event: {e:?}");
        handle_event(stg, &e);
    }
}

#[derive(SystemParam)]
struct StageQueries<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    asset_server: Res<'w, AssetServer>,
    stage: Query<'w, 's, (Entity, &'static mut GuiStage)>,
    players: Query<'w, 's, (Entity, &'static mut GuiPlayer)>,
    hands: Query<'w, 's, (Entity, &'static mut GuiHand)>,
    melds: Query<'w, 's, (Entity, &'static mut GuiMelds)>,
    tiles: Query<'w, 's, (Entity, &'static mut GuiTile)>,
}

fn handle_event(stg: StageQueries, event: &model::Event) {
    match event {
        model::Event::Begin(e) => event_begin(stg, e),
        model::Event::New(e) => event_new(stg, e),
        model::Event::Deal(e) => event_deal(stg, e),
        model::Event::Discard(e) => event_discard(stg, e),
        model::Event::Meld(e) => event_meld(stg, e),
        model::Event::Nukidora(e) => event_nukidora(stg, e),
        model::Event::Dora(e) => event_dora(stg, e),
        model::Event::Win(e) => event_win(stg, e),
        model::Event::Draw(e) => event_draw(stg, e),
        model::Event::End(e) => event_end(stg, e),
    }
}

fn event_begin(stg: StageQueries, event: &model::EventBegin) {}

fn event_new(stg: StageQueries, event: &model::EventNew) {}

fn event_deal(stg: StageQueries, event: &model::EventDeal) {}

fn event_discard(stg: StageQueries, event: &model::EventDiscard) {}

fn event_meld(stg: StageQueries, event: &model::EventMeld) {}

fn event_nukidora(stg: StageQueries, event: &model::EventNukidora) {}

fn event_dora(stg: StageQueries, event: &model::EventDora) {}

fn event_win(stg: StageQueries, event: &model::EventWin) {}

fn event_draw(stg: StageQueries, event: &model::EventDraw) {}

fn event_end(stg: StageQueries, event: &model::EventEnd) {}
