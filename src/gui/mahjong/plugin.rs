use std::sync::Mutex;

use super::{
    param::{ActionParam, MahjongParam, with_param},
    prelude::*,
    setting::{SettingParam, SettingProps},
    stage::GuiStage,
    tile_plugin::TilePlugin,
};

pub type Tx = std::sync::mpsc::Sender<ClientMessage>;
pub type Rx = std::sync::mpsc::Receiver<ServerMessage>;

#[derive(Resource)]
pub struct InfoTexture(pub Handle<Image>);

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
        app.add_plugins(TilePlugin)
            .insert_resource(GuiMahjong::new(tx, rx))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    mahjong_process_server_message,
                    mahjong_handle_action_events,
                    mahjong_handle_setting_events,
                )
                    .chain(),
            );
    }
}

fn setup(mut cmd: Commands, mut images: ResMut<Assets<Image>>) {
    // 中央パネルのテクスチャを初期化
    // レンダリング用のカメラより先に初期化される必要があるためここで実行
    // 公式のExamplesでは同時に初期化しているが多くの初期化処理を一度に行う場合に正しく動作しない
    use bevy::render::render_resource::TextureFormat;
    let image = Image::new_target_texture(512, 512, TextureFormat::bevy_default());
    cmd.insert_resource(InfoTexture(images.add(image)));
}

fn mahjong_process_server_message(mut gui_mahjong: ResMut<GuiMahjong>, mut param: MahjongParam) {
    with_param(&mut param, || {
        gui_mahjong.process_server_message();
    });
}

fn mahjong_handle_action_events(
    mut gui_mahjong: ResMut<GuiMahjong>,
    mut param: MahjongParam,
    mut action_param: ActionParam,
) {
    with_param(&mut param, || {
        gui_mahjong.handle_action_events(&mut action_param);
    });
}

fn mahjong_handle_setting_events(
    mut gui_mahjong: ResMut<GuiMahjong>,
    mut param: MahjongParam,
    mut setting_param: SettingParam,
) {
    with_param(&mut param, || {
        gui_mahjong.handle_setting_events(&mut setting_param);
    });
}

#[derive(Resource, Debug)]
struct GuiMahjong {
    stage: Option<GuiStage>,
    player_seat: Option<Seat>,
    tx: Mutex<Tx>,
    rx: Mutex<Rx>,
}

impl GuiMahjong {
    fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            stage: None,
            player_seat: None,
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }

    fn process_server_message(&mut self) {
        while let Ok(msg) = self.rx.lock().unwrap().try_recv() {
            if let ServerMessage::Event(ev) = &msg
                && let MjEvent::New(_) = ev.as_ref()
            {
                let mut props = SettingProps {
                    show_wall: false,
                    show_hand: false,
                    camera_seat: self.player_seat.unwrap_or(0),
                };
                // ステージ上のentityを再帰的にすべて削除
                if let Some(stage) = self.stage.take() {
                    props = stage.get_setting_props().clone();
                    stage.destroy();
                }

                // 空のステージを作成
                let mut stage = GuiStage::new();
                if let Some(s) = self.player_seat {
                    stage.set_player(s);
                }
                stage.set_setting_props(props);
                self.stage = Some(stage);
            }

            match msg {
                ServerMessage::Event(event) => {
                    match event.as_ref() {
                        MjEvent::Begin(_ev) => {}
                        MjEvent::End(_ev) => {}
                        _ => self.stage.as_mut().unwrap().handle_event(&event),
                    }

                    // TODO
                    // 一度のUpdateで複数のEventの更新を行うとGlobalTransformに
                    // GuiTileのentityが追加される前にreparent_transformsformから
                    // Query::get()が呼び出され失敗する
                    break;
                }
                ServerMessage::Action(possible_actions) => {
                    // Nopのみしか選択肢がない場合は即時応答
                    if possible_actions.actions.len() == 1
                        && possible_actions.actions[0].ty == ActionType::Nop
                    {
                        self.tx
                            .lock()
                            .unwrap()
                            .send(ClientMessage::Action(SelectedAction {
                                id: possible_actions.id,
                                action: Action::nop(),
                            }))
                            .unwrap();
                        break;
                    }

                    self.stage
                        .as_mut()
                        .unwrap()
                        .handle_actions(possible_actions);
                }
                ServerMessage::Info { seat } => {
                    self.player_seat = Some(seat); // このメッセージはEventNewより先に受信
                }
                ServerMessage::Log => todo!(),
            }
        }
    }

    fn handle_action_events(&mut self, action_param: &mut ActionParam) {
        if let Some(stage) = &mut self.stage
            && let Some(act) = stage.handle_action_events(action_param)
        {
            self.tx
                .lock()
                .unwrap()
                .send(ClientMessage::Action(act))
                .unwrap();
        }

        // 使用されなかったbevyのMessageは次のフレームには持ち越されない
    }

    fn handle_setting_events(&mut self, setting_param: &mut SettingParam) {
        if let Some(stage) = &mut self.stage {
            stage.handle_setting_events(setting_param);
        }
    }
}
