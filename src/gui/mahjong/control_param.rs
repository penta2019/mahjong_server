use std::thread::{self, ThreadId};

use bevy::{ecs::system::SystemParam, input::mouse::MouseButtonInput, prelude::*};

use super::{
    super::{camera::CameraMove, util::print_hierarchy},
    action::GameButton,
    tile_plugin::{HoveredTile, TileControl},
};

// stage_plugin::stage_main_loopから呼び出される関数(controlディレクトリのモジュール)から使用するパラメータ
#[derive(SystemParam)]
pub struct ControlParam<'w, 's> {
    pub commands: Commands<'w, 's>,
    // pub window: Single<'w, &'static mut Window>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
    pub globals: Query<'w, 's, &'static mut GlobalTransform>,
    pub tile_controls: Query<'w, 's, &'static mut TileControl>,
    pub game_buttons: Query<
        'w,
        's,
        (
            &'static Interaction,
            &'static mut GameButton,
            &'static mut BorderColor,
        ),
        Changed<Interaction>,
    >,

    // MessageWriter
    pub camera: MessageWriter<'w, CameraMove>,

    // MessageReader
    pub hovered_tile: MessageReader<'w, 's, HoveredTile>,
    pub mouse_input: MessageReader<'w, 's, MouseButtonInput>,

    // for debug
    pub names: Query<'w, 's, &'static Name>,
    pub childrens: Query<'w, 's, &'static Children>,
}

impl<'w, 's> ControlParam<'w, 's> {
    #[allow(unused)]
    pub fn print_hierarchy(&self, entity: Entity) {
        print_hierarchy(entity, &self.names, &self.childrens);
    }

    pub fn drain_events(&mut self) {
        self.hovered_tile.read();
        self.mouse_input.read();
    }
}

// StageParamをすべての関数にたらい回しにするのはあまりに冗長であるためグローバル変数を使用
// 注意!!!!
// * process_event以下の関数以外からは呼ばないこと,特にadd_systemsに登録する関数に注意
// * 戻り値のStageParamの参照を関数から返したり,ローカル変数以外に格納しないこと
// * lifetimeを誤魔化しているため,borrow checkerは正しく機能しない
//      (= 呼び出した関数内で状態が変化する可能性がある)
static mut CONTROL_PARAM: Option<(*mut (), ThreadId)> = None;
pub(super) fn param<'w, 's>() -> &'static mut ControlParam<'w, 's> {
    unsafe {
        let (p, tid) = CONTROL_PARAM.unwrap();
        // 誤って別スレッドからアクセスして未定義の振る舞いを起こすのを防止
        assert!(tid == thread::current().id());
        let p = p as *mut ControlParam<'w, 's>;
        &mut *p
    }
}

// 関数fの実行中にparam()から&mut GuiStageを取得できるよう設定
pub fn with_param<F: FnOnce()>(param: &mut ControlParam, f: F) {
    let ptr_param = param as *mut ControlParam as *mut ();
    let tid = thread::current().id();

    unsafe {
        // 同じSystemParamを参照する関数は同時実行されないはずだが念の為
        #[allow(static_mut_refs)]
        let safe_to_use = CONTROL_PARAM.is_none();
        assert!(safe_to_use);

        CONTROL_PARAM = Some((ptr_param, tid));
        f();
        CONTROL_PARAM = None;
    };
}
