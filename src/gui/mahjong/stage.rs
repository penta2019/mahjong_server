use std::{
    sync::Mutex,
    thread::{self, ThreadId},
};

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

#[derive(SystemParam)]
pub struct StageParam<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
    pub globals: Query<'w, 's, &'static mut GlobalTransform>,
    // for debug
    pub names: Query<'w, 's, &'static Name>,
    pub children: Query<'w, 's, &'static Children>,
}

// StageParamをすべての関数にたらい回しにするのはあまりに冗長であるためグローバル変数を使用
// 注意!!!!
// * process_event以下の関数以外からは呼ばないこと,特にadd_systemsに登録する関数に注意
// * 戻り値のStageParamの参照を関数から返したり,ローカル変数以外に格納しないこと
static mut STAGE_PARAM: Option<(*mut (), ThreadId)> = None;
pub(super) fn param<'w, 's>() -> &'static mut StageParam<'w, 's> {
    unsafe {
        let (p, tid) = STAGE_PARAM.unwrap();
        assert!(tid == thread::current().id());
        let p = p as *mut StageParam<'w, 's>;
        &mut *p
    }
}

fn process_event(
    mut param: StageParam,
    mut stage: ResMut<GuiStage>,
    event_reader: ResMut<EventReceiver>,
) {
    let param = &mut param as *mut StageParam as *mut ();
    let tid = thread::current().id();
    unsafe { STAGE_PARAM = Some((param, tid)) };

    if let Ok(event) = event_reader.recv.lock().unwrap().try_recv() {
        handle_event(&mut stage, &event);
    }

    unsafe { STAGE_PARAM = None };
}

fn handle_event(stage: &mut GuiStage, event: &MjEvent) {
    match &event {
        MjEvent::Begin(_ev) => {}
        MjEvent::New(ev) => {
            // ステージ上のentityを再帰的にすべて削除
            param().commands.entity(stage.entity()).despawn();
            // 空のステージを作成
            *stage = GuiStage::new();

            stage.event_new(ev);
        }
        MjEvent::Deal(ev) => stage.event_deal(ev),
        MjEvent::Discard(ev) => stage.event_discard(ev),
        // MjEvent::Meld(ev) => stage.event_meld(ev),
        // MjEvent::Nukidora(ev) => stage.event_nukidora(ev),
        // MjEvent::Dora(ev) => stage.event_dora(ev),
        // MjEvent::Win(ev) => stage.event_win(ev),
        // MjEvent::Draw(ev) => stage.event_draw(ev),
        MjEvent::End(_ev) => {}
        _ => {}
    }
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

    fn new() -> Self {
        let param = param();
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
            stage.players.push(GuiPlayer::new(e_stage, seat));
        }

        stage
    }

    fn event_new(&mut self, event: &EventNew) {
        for seat in 0..SEAT {
            self.players[seat].init_hand(&event.hands[seat]);
        }
    }

    fn event_deal(&mut self, event: &EventDeal) {
        self.players[event.seat].deal_tile(event.tile);
    }

    fn event_discard(&mut self, event: &EventDiscard) {
        self.players[event.seat].discard_tile(event.tile, event.is_drawn, event.is_riichi);
    }

    // fn event_meld(&mut self, event: &EventMeld) {}

    // fn event_nukidora(&mut self, event: &EventNukidora) {}

    // fn event_dora(&mut self, event: &EventDora) {}

    // fn event_win(&mut self, event: &EventWin) {}

    // fn event_draw(&mut self, event: &EventDraw) {}
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
