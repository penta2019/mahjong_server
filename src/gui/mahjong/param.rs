use std::thread::{self, ThreadId};

use bevy::{ecs::system::SystemParam, input::mouse::MouseButtonInput, prelude::*};

use super::{
    super::{camera::CameraMove, util::print_hierarchy},
    action::GameButton,
    plugin::InfoTexture,
    setting::SettingButton,
    tile_plugin::HoveredTile,
};

// stage_plugin::stage_main_loopから呼び出される関数(controlディレクトリのモジュール)から使用するパラメータ
#[derive(SystemParam)]
pub struct MahjongParam<'w, 's> {
    pub cmd: Commands<'w, 's>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub asset_server: Res<'w, AssetServer>,
    pub globals: Query<'w, 's, &'static mut GlobalTransform>,
    // for debug
    pub names: Query<'w, 's, &'static Name>,
    pub childrens: Query<'w, 's, &'static Children>,
    // 中央情報パネルのテクスチャ
    pub info_texture: Res<'w, InfoTexture>,
    // カメラの移動
    pub camera: MessageWriter<'w, CameraMove>,
}

impl<'w, 's> MahjongParam<'w, 's> {
    #[allow(unused)]
    pub fn print_hierarchy(&self, entity: Entity) {
        print_hierarchy(entity, &self.names, &self.childrens);
    }
}

// MahjongParamをすべての関数にたらい回しにするのはあまりに冗長であるためグローバル変数を使用
// 注意!!!!
// * GuiMahjongとその子孫以外からは呼ばないこと,特にadd_systemsに登録する関数に注意
// * 戻り値のMahjongParamの参照を関数から返したり,ローカル変数以外に格納しないこと
// * lifetimeを誤魔化しているため,borrow checkerは正しく機能しない
//      (= 呼び出した関数内で状態が変化する可能性がある)
static mut MAHJONG_PARAM: Option<(*mut (), ThreadId)> = None;
pub fn param<'w, 's>() -> &'static mut MahjongParam<'w, 's> {
    unsafe {
        let (p, tid) = MAHJONG_PARAM.unwrap();
        // 誤って別スレッドからアクセスして未定義の振る舞いを起こすのを防止
        assert!(tid == thread::current().id());
        let p = p as *mut MahjongParam<'w, 's>;
        &mut *p
    }
}

#[inline]
pub fn cmd<'w, 's>() -> &'static mut Commands<'w, 's> {
    &mut param().cmd
}

// 関数fの実行中にparam()から&mut GuiStageを取得できるよう設定
pub fn with_param<F: FnOnce()>(param: &mut MahjongParam, f: F) {
    let ptr_param = param as *mut MahjongParam as *mut ();
    let tid = thread::current().id();

    unsafe {
        // 同じSystemParamを参照する関数は同時実行されないはずだが念の為
        #[allow(static_mut_refs)]
        let safe_to_use = MAHJONG_PARAM.is_none();
        assert!(safe_to_use);

        MAHJONG_PARAM = Some((ptr_param, tid));
        f();
        MAHJONG_PARAM = None;
    };
}

#[derive(SystemParam)]
pub struct ActionParam<'w, 's> {
    // Game Button
    pub game_buttons: Query<
        'w,
        's,
        (
            &'static mut GameButton,
            &'static mut BorderColor,
            &'static mut BackgroundColor,
        ),
    >, // 実装側から参照できるようにInteractionから分離
    pub game_button_interactions:
        Query<'w, 's, (Entity, &'static Interaction), (Changed<Interaction>, With<GameButton>)>,

    // MessageReader
    pub hovered_tile: MessageReader<'w, 's, HoveredTile>,
    pub mouse_input: MessageReader<'w, 's, MouseButtonInput>,
}

#[derive(SystemParam)]
pub struct SettingParam<'w, 's> {
    // Setting Button
    pub setting_buttons: Query<
        'w,
        's,
        (
            &'static mut SettingButton,
            &'static mut BorderColor,
            &'static mut BackgroundColor,
        ),
    >,
    // pub setting_button_interactions:
    //     Query<'w, 's, (Entity, &'static Interaction), (Changed<Interaction>, With<SettingButton>)>,
}
