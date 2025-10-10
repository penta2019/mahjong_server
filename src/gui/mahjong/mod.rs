mod action;
mod control;
mod control_param;
mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod stage_info;
mod tile;
mod tile_plugin;

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::prelude::*;

use self::{
    action::ActionControl,
    control::MahjonGuiControl,
    control_param::{ControlParam, param, with_param},
    discard::GuiDiscard,
    hand::{GuiHand, IsDrawn},
    meld::GuiMeld,
    player::{GuiPlayer, HandMode},
    stage::GuiStage,
    stage_info::StageInfo,
    tile::{GuiTile, TILE_ACTIVE, TILE_INACTIVE, TILE_NORMAL},
};
use crate::model::{Event as MjEvent, *};

pub type Tx = std::sync::mpsc::Sender<ClientMessage>;
pub type Rx = std::sync::mpsc::Receiver<ServerMessage>;

#[derive(Resource)]
struct InfoTexture(Handle<Image>);

trait HasEntity {
    fn entity(&self) -> Entity;
}

pub struct MahjongPlugin {
    txrx: std::sync::Mutex<Option<(Tx, Rx)>>,
}

impl MahjongPlugin {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            txrx: std::sync::Mutex::new(Some((tx, rx))),
        }
    }
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = self.txrx.lock().unwrap().take().unwrap();
        app.add_plugins(tile_plugin::TilePlugin)
            .insert_resource(MahjonGuiControl::new(tx, rx))
            .add_systems(Startup, setup)
            .add_systems(Update, mahjong_control);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // 中央パネルのテクスチャを初期化
    // レンダリング用のカメラより先に初期化される必要があるためここで実行
    // 公式のExamplesでは同時に初期化しているが多くの初期化処理を一度に行う場合に正しく動作しない
    use bevy::render::render_resource::TextureFormat;
    let image = Image::new_target_texture(512, 512, TextureFormat::bevy_default());
    commands.insert_resource(InfoTexture(images.add(image)));
}

fn mahjong_control(mut param: ControlParam, mut stage_control: ResMut<MahjonGuiControl>) {
    with_param(&mut param, || {
        stage_control.update();
    });
}
