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
    // pub names: Query<'w, 's, &'static Name>,
    // pub children: Query<'w, 's, &'static Children>,
}

// StageParamをすべての関数にたらい回しにするのはあまりに冗長であるためグローバル変数を使用
// 注意!!!!
// * process_event以下の関数以外からは呼ばないこと,特にadd_systemsに登録する関数に注意
// * 戻り値のStageParamの参照を関数から返したり,ローカル変数以外に格納しないこと
static mut STAGE_PARAM: Option<(*mut (), ThreadId)> = None;
pub(super) fn param<'w, 's>() -> &'static mut StageParam<'w, 's> {
    unsafe {
        let (p, tid) = STAGE_PARAM.unwrap();
        // 誤って別スレッドからアクセスして未定義の振る舞いを起こすのを防止
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
    // この関数の実行中にparam()から&mut GuiStageを取得できるよう設定
    let param = &mut param as *mut StageParam as *mut ();
    let tid = thread::current().id();
    unsafe { STAGE_PARAM = Some((param, tid)) };

    if let Ok(event) = event_reader.recv.lock().unwrap().try_recv() {
        handle_event(&mut stage, &event);
    }

    // この関数外から呼ばれて未定義の振る舞いを起こすのを防止
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
        MjEvent::Meld(ev) => stage.event_meld(ev),
        MjEvent::Nukidora(ev) => stage.event_nukidora(ev),
        MjEvent::Dora(ev) => stage.event_dora(ev),
        MjEvent::Win(ev) => stage.event_win(ev),
        MjEvent::Draw(ev) => stage.event_draw(ev),
        MjEvent::End(_ev) => {}
    }
}

#[derive(Resource, Debug)]
struct GuiStage {
    entity: Entity,
    players: Vec<GuiPlayer>,
    last_tile: Option<(Seat, ActionType, Tile)>,
}

impl GuiStage {
    fn empty() -> Self {
        GuiStage {
            entity: Entity::PLACEHOLDER,
            players: vec![],
            last_tile: None,
        }
    }

    fn new() -> Self {
        let param = param();
        let commands = &mut param.commands;
        let meshes = &mut param.meshes;
        let materials = &mut param.materials;

        // stage
        let entity = commands
            .spawn((
                Name::new("Stage".to_string()),
                Transform::from_xyz(0., 0., 0.),
                Mesh3d(meshes.add(Plane3d::default().mesh().size(0.65, 0.65))),
                MeshMaterial3d(materials.add(Color::from(GREEN))),
            ))
            .id();

        // info
        commands.spawn((
            Name::new("Info".to_string()),
            ChildOf(entity),
            Transform::from_xyz(0., 0.001, 0.),
            Mesh3d(meshes.add(Plane3d::default().mesh().size(0.12, 0.12))),
            MeshMaterial3d(materials.add(Color::from(BLACK))),
        ));

        // Light
        commands.spawn((
            ChildOf(entity),
            PointLight {
                shadows_enabled: true,
                intensity: 10_000_000.,
                range: 100.0,
                shadow_depth_bias: 0.2,
                ..default()
            },
            Transform::from_xyz(8.0, 16.0, 8.0),
        ));

        let mut players = vec![];
        for seat in 0..SEAT {
            let player = GuiPlayer::new();
            commands.entity(player.entity()).insert((
                ChildOf(entity),
                Transform {
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 * seat as f32),
                    ..Default::default()
                },
            ));
            players.push(player);
        }

        Self {
            entity,
            players,
            last_tile: None,
        }
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
        self.last_tile = Some((event.seat, ActionType::Discard, event.tile));
    }

    fn event_meld(&mut self, event: &EventMeld) {
        // 鳴いたプレイヤーから半時計回りに見た牌を捨てたプレイヤーの座席
        // 自身(0), 下家(1), 対面(2), 上家(3)
        let mut meld_offset = 0;

        // 他家が捨てた牌
        let meld_tile = match event.meld_type {
            MeldType::Chi | MeldType::Pon | MeldType::Minkan => {
                let target_seat = self.last_tile.unwrap().0;
                meld_offset = (target_seat + SEAT - event.seat) % SEAT;
                Some(self.players[target_seat].take_last_discard_tile())
            }
            _ => None,
        };

        self.players[event.seat].meld(&event.consumed, meld_tile, meld_offset);
    }

    fn event_nukidora(&mut self, _event: &EventNukidora) {}

    fn event_dora(&mut self, _event: &EventDora) {}

    fn event_win(&mut self, _event: &EventWin) {}

    fn event_draw(&mut self, _event: &EventDraw) {}
}

impl HasEntity for GuiStage {
    fn entity(&self) -> Entity {
        self.entity
    }
}
