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
    model::{self, *},
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
    tiles: Vec<(Tile, Entity)>,
}

#[derive(Component, Debug)]
pub struct GuiMelds {
    seat: Seat,
}

#[derive(SystemParam)]
struct StageParam<'w, 's> {
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

fn setup() {

    // for i in 0..13 {
    //     let tile = super::tile::create_tile(&mut commands, &asset_server, Tile(model::TM, 5));
    //     commands.entity(tile).insert((
    //         ChildOf(hand),
    //         Transform::from_xyz(TILE_WIDTH * i as f32, TILE_HEIGHT / 2., 0.01 as f32),
    //     ));
    // }
}

fn read_event(param: StageParam, event_reader: ResMut<EventReceiver>) {
    if let Ok(e) = event_reader.recv.lock().unwrap().try_recv() {
        handle_event(param, &e);
    }
}

fn init_stage(mut param: StageParam) {
    for (entity, _) in &param.stage {
        param.commands.entity(entity).despawn();
    }

    // Light
    param.commands.spawn((
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
    let stage = param
        .commands
        .spawn((
            Transform::from_xyz(0., 0., 0.),
            Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
            MeshMaterial3d(param.materials.add(Color::from(GREEN))),
            GuiStage,
        ))
        .id();

    // info
    param.commands.spawn((
        ChildOf(stage),
        Transform::from_xyz(0., 0.001, 0.),
        Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
        MeshMaterial3d(param.materials.add(Color::from(BLACK))),
    ));

    for s in 0..SEAT {
        let player = param
            .commands
            .spawn((
                ChildOf(stage),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * s as f32),
                    ..Default::default()
                },
                GuiPlayer { seat: s },
            ))
            .id();

        param.commands.spawn((
            ChildOf(player),
            Transform::from_xyz(-0.12, 0., 0.21),
            GuiHand {
                seat: s,
                tiles: vec![],
            },
        ));
    }
}

fn handle_event(param: StageParam, event: &model::Event) {
    match event {
        model::Event::Begin(e) => event_begin(param, e),
        model::Event::New(e) => event_new(param, e),
        model::Event::Deal(e) => event_deal(param, e),
        model::Event::Discard(e) => event_discard(param, e),
        model::Event::Meld(e) => event_meld(param, e),
        model::Event::Nukidora(e) => event_nukidora(param, e),
        model::Event::Dora(e) => event_dora(param, e),
        model::Event::Win(e) => event_win(param, e),
        model::Event::Draw(e) => event_draw(param, e),
        model::Event::End(e) => event_end(param, e),
    }
}

fn event_begin(param: StageParam, _event: &model::EventBegin) {
    init_stage(param);
}

fn event_new(param: StageParam, event: &model::EventNew) {
    for seat in 0..SEAT {
        for tile in &event.hands[seat] {
            for (e_hand, hand) in &param.hands {}
        }
    }
}

fn event_deal(param: StageParam, event: &model::EventDeal) {}

fn event_discard(param: StageParam, event: &model::EventDiscard) {}

fn event_meld(param: StageParam, event: &model::EventMeld) {}

fn event_nukidora(param: StageParam, event: &model::EventNukidora) {}

fn event_dora(param: StageParam, event: &model::EventDora) {}

fn event_win(param: StageParam, event: &model::EventWin) {}

fn event_draw(param: StageParam, event: &model::EventDraw) {}

fn event_end(param: StageParam, event: &model::EventEnd) {}
