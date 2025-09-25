use std::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self, ThreadId},
};

use bevy::{
    ecs::system::SystemParam,
    input::{
        ButtonState,
        mouse::{MouseButtonInput, MouseMotion},
    },
};

use super::{
    super::{
        control::{CameraEvent, MainCamera},
        util::print_hierarchy,
    },
    tile::{TileMesh, TileMutateEvent},
    *,
};
use crate::model::{Event as MjEvent, *};

pub type Tx = Sender<ClientMessage>;
pub type Rx = Receiver<ServerMessage>;

pub struct StagePlugin {
    event_rx: Mutex<Option<(Tx, Rx)>>,
}

impl StagePlugin {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            event_rx: Mutex::new(Some((tx, rx))),
        }
    }
}

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = self.event_rx.lock().unwrap().take().unwrap();
        app.insert_resource(StageResource::new(tx, rx))
            .add_event::<TileHoverEvent>()
            .add_systems(Update, handle_mouse_motion)
            .add_systems(Update, process_event);
    }
}

#[derive(Event, Debug)]
struct TileHoverEvent {
    tile_entity: Option<Entity>,
}

#[derive(SystemParam)]
pub struct StageParam<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
    pub globals: Query<'w, 's, &'static mut GlobalTransform>,
    pub camera: EventWriter<'w, CameraEvent>,
    pub tile_mutate: EventWriter<'w, TileMutateEvent>,
    pub tile_tags: Query<'w, 's, &'static TileTag>,

    // for debug
    pub names: Query<'w, 's, &'static Name>,
    pub childrens: Query<'w, 's, &'static Children>,
}

impl<'w, 's> StageParam<'w, 's> {
    #[allow(unused)]
    pub fn print_hierarchy(&self, entity: Entity) {
        print_hierarchy(entity, &self.names, &self.childrens);
    }
}

#[derive(Resource, Debug)]
pub struct StageResource {
    stage: Option<GuiStage>,
    seat: Seat,
    tx: Mutex<Tx>,
    rx: Mutex<Rx>,
}

impl StageResource {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            stage: None,
            seat: 0,
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }

    pub fn update(&mut self) {
        while let Ok(msg) = self.rx.lock().unwrap().try_recv() {
            if let ServerMessage::Event(ev) = &msg
                && let MjEvent::New(_) = ev.as_ref()
            {
                // ステージ上のentityを再帰的にすべて削除
                if let Some(stage) = self.stage.take() {
                    param().commands.entity(stage.entity()).despawn();
                }

                // 空のステージを作成
                let mut stage = GuiStage::new();
                stage.set_player(self.seat);
                self.stage = Some(stage);
            }

            match msg {
                ServerMessage::Event(event) => {
                    let Some(stage) = self.stage.as_mut() else {
                        continue;
                    };

                    match event.as_ref() {
                        MjEvent::Begin(_ev) => {}
                        MjEvent::New(ev) => stage.event_new(ev),
                        MjEvent::Deal(ev) => stage.event_deal(ev),
                        MjEvent::Discard(ev) => stage.event_discard(ev),
                        MjEvent::Meld(ev) => stage.event_meld(ev),
                        MjEvent::Nukidora(ev) => stage.event_nukidora(ev),
                        MjEvent::Dora(ev) => stage.event_dora(ev),
                        MjEvent::Win(ev) => stage.event_win(ev),
                        MjEvent::Draw(ev) => stage.event_draw(ev),
                        MjEvent::End(_ev) => {}
                    }

                    // TODO
                    // 一度のUpdateで複数のEventの更新を行うとGlobalTransformに
                    // GuiTileのentityが追加される前にget()が呼び出され失敗する
                    break;
                }
                ServerMessage::Action {
                    id,
                    actions,
                    tenpais,
                } => {
                    let action = ClientMessage::Action {
                        id,
                        action: Action::nop(),
                    };
                    self.tx.lock().unwrap().send(action).unwrap();
                }
                ServerMessage::Info { seat } => {
                    self.seat = seat; // このメッセージはEventNewより先に受信
                }
            }
        }

        // param().print_hierarchy(stage.entity());
    }

    pub fn set_hover_tile(&mut self, entity: Option<Entity>) {
        if let Some(stage) = self.stage.as_mut() {
            stage.set_hover_tile(entity);
        }
    }
}

// StageParamをすべての関数にたらい回しにするのはあまりに冗長であるためグローバル変数を使用
// 注意!!!!
// * process_event以下の関数以外からは呼ばないこと,特にadd_systemsに登録する関数に注意
// * 戻り値のStageParamの参照を関数から返したり,ローカル変数以外に格納しないこと
// * lifetimeを誤魔化しているため,borrow checkerは正しく機能しない
//      (= 呼び出した関数内で状態が変化する可能性がある)
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

// 関数fの実行中にparam()から&mut GuiStageを取得できるよう設定
fn with_param<F: FnOnce()>(param: &mut StageParam, f: F) {
    let ptr_param = param as *mut StageParam as *mut ();
    let tid = thread::current().id();

    unsafe {
        // 同じSystemParamを参照する関数は同時実行されないはずだが念の為
        #[allow(static_mut_refs)]
        let safe_to_use = STAGE_PARAM.is_none();
        assert!(safe_to_use);

        STAGE_PARAM = Some((ptr_param, tid));
        f();
        STAGE_PARAM = None;
    };
}

pub fn create_tile(m_tile: Tile) -> GuiTile {
    let param = param();
    GuiTile::new(&mut param.commands, &param.asset_server, m_tile)
}

fn handle_mouse_motion(
    mut mouse_events: EventReader<MouseMotion>,
    window: Single<&mut Window>,
    camera: Single<(&mut Camera, &GlobalTransform), With<MainCamera>>,
    mut ray_cast: MeshRayCast,
    tile_meshes: Query<&TileMesh>,
    mut tile_hover: EventWriter<TileHoverEvent>,
) {
    let Some(_) = mouse_events.read().next() else {
        return;
    };
    let Some(p_cursor) = window.cursor_position() else {
        return;
    };
    let (camera, tf_camera) = &*camera;
    let Ok(ray) = camera.viewport_to_world(tf_camera, p_cursor) else {
        return;
    };

    for (entity, _hit) in ray_cast.cast_ray(ray, &MeshRayCastSettings::default()) {
        if let Ok(m) = tile_meshes.get(*entity) {
            tile_hover.write(TileHoverEvent {
                tile_entity: Some(m.tile_entity()),
            });
            return;
        }
    }
    tile_hover.write(TileHoverEvent { tile_entity: None });
}

fn process_event(
    mut param: StageParam,
    mut stage_res: ResMut<StageResource>,
    mut tile_hover: EventReader<TileHoverEvent>,
    mut mouse_input: EventReader<MouseButtonInput>,
) {
    with_param(&mut param, || {
        stage_res.update();

        for ev in tile_hover.read() {
            stage_res.set_hover_tile(ev.tile_entity);
        }

        for ev in mouse_input.read() {
            match ev.state {
                ButtonState::Pressed => {
                    println!("Mouse button press: {:?}", ev.button);
                }
                ButtonState::Released => {
                    println!("Mouse button release: {:?}", ev.button);
                }
            }
        }
    });
}
