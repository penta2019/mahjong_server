use std::sync::Mutex;

use bevy::{
    color::palettes::basic::{BLACK, GREEN},
    ecs::system::SystemParam,
};

use super::*;
use crate::{
    listener::EventRx,
    model::{Event as MjEvent, *},
};

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
        .insert_resource(GuiStage::empty())
        .add_systems(Update, process_event);
    }
}

#[derive(Resource, Debug)]
struct EventReceiver {
    recv: Mutex<EventRx>,
}

fn process_event(
    mut stage: ResMut<GuiStage>,
    mut param: StageParam,
    event_reader: ResMut<EventReceiver>,
) {
    if let Ok(event) = event_reader.recv.lock().unwrap().try_recv() {
        let param = &mut param;
        match &event {
            MjEvent::Begin(_ev) => {}
            MjEvent::New(ev) => {
                // ステージ上のentityを再帰的にすべて削除
                param.commands.entity(stage.entity()).despawn();
                // 空のステージを作成
                *stage = GuiStage::new(param);

                stage.event_new(param, ev);
            }
            MjEvent::Deal(ev) => stage.event_deal(param, ev),
            MjEvent::Discard(ev) => stage.event_discard(param, ev),
            // MjEvent::Meld(ev) => stage.event_meld(param, ev),
            // MjEvent::Nukidora(ev) => stage.event_nukidora(param, ev),
            // MjEvent::Dora(ev) => stage.event_dora(param, ev),
            // MjEvent::Win(ev) => stage.event_win(param, ev),
            // MjEvent::Draw(ev) => stage.event_draw(param, ev),
            MjEvent::End(_ev) => {}
            _ => {}
        }
    }
}

#[derive(SystemParam)]
pub struct StageParam<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
    pub globals: Query<'w, 's, &'static mut GlobalTransform>,
    // for debug
    // names: Query<'w, 's, &'static Name>,
    // children: Query<'w, 's, &'static Children>,
}

#[derive(Resource, Debug)]
struct GuiStage {
    entity: Entity,
    info: GuiInfo,
    players: Vec<GuiPlayer>,
}

impl GuiStage {
    fn empty() -> Self {
        GuiStage {
            entity: Entity::PLACEHOLDER,
            info: GuiInfo {
                entity: Entity::PLACEHOLDER,
            },
            players: vec![],
        }
    }

    fn new(param: &mut StageParam) -> Self {
        let commands = &mut param.commands;

        // stage
        let e_stage = commands
            .spawn((
                Name::new("Stage".to_string()),
                Transform::from_xyz(0., 0., 0.),
                Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
                MeshMaterial3d(param.materials.add(Color::from(GREEN))),
            ))
            .id();

        // info
        let e_info = commands
            .spawn((
                Name::new("Info".to_string()),
                ChildOf(e_stage),
                Transform::from_xyz(0., 0.001, 0.),
                Mesh3d(param.meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
                MeshMaterial3d(param.materials.add(Color::from(BLACK))),
            ))
            .id();

        // Light
        commands.spawn((
            ChildOf(e_stage),
            PointLight {
                shadows_enabled: true,
                intensity: 10_000_000.,
                range: 100.0,
                shadow_depth_bias: 0.2,
                ..default()
            },
            Transform::from_xyz(8.0, 16.0, 8.0),
        ));

        let mut stage = GuiStage {
            entity: e_stage,
            info: GuiInfo { entity: e_info },
            players: vec![],
        };

        for seat in 0..SEAT {
            stage.players.push(GuiPlayer::new(param, e_stage, seat));
        }

        stage
    }

    fn event_new(&mut self, param: &mut StageParam, event: &EventNew) {
        let commands = &mut param.commands;
        for seat in 0..SEAT {
            self.players[seat].init_hand(param, &event.hands[seat]);
        }
    }

    fn event_deal(&mut self, param: &mut StageParam, event: &EventDeal) {
        self.players[event.seat].deal_tile(param, event.tile);
    }

    fn event_discard(&mut self, param: &mut StageParam, event: &EventDiscard) {
        self.players[event.seat].discard_tile(param, event.tile, event.is_drawn, event.is_riichi);
    }

    // fn event_meld(&mut self, param: &mut StageParam, event: &EventMeld) {}

    // fn event_nukidora(&mut self, param: &mut StageParam, event: &EventNukidora) {}

    // fn event_dora(&mut self, param: &mut StageParam, event: &EventDora) {}

    // fn event_win(&mut self, param: &mut StageParam, event: &EventWin) {}

    // fn event_draw(&mut self, param: &mut StageParam, event: &EventDraw) {}
}

impl HasEntity for GuiStage {
    fn entity(&self) -> Entity {
        self.entity
    }
}

#[derive(Debug)]
struct GuiInfo {
    entity: Entity,
}
