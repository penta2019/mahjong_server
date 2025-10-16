use std::sync::Mutex;

use super::{
    param::{MahjongParam, with_param},
    prelude::*,
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
            .add_systems(Update, mahjong_control);
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

fn mahjong_control(mut param: MahjongParam, mut stage_control: ResMut<GuiMahjong>) {
    with_param(&mut param, || {
        stage_control.update();
    });
}

#[derive(Resource, Debug)]
struct GuiMahjong {
    stage: Option<GuiStage>,
    seat: Seat,
    tx: Mutex<Tx>,
    rx: Mutex<Rx>,
}

impl GuiMahjong {
    fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            stage: None,
            seat: 0,
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
        }
    }

    fn update(&mut self) {
        while let Ok(msg) = self.rx.lock().unwrap().try_recv() {
            if let ServerMessage::Event(ev) = &msg
                && let MjEvent::New(_) = ev.as_ref()
            {
                // ステージ上のentityを再帰的にすべて削除
                if let Some(stage) = self.stage.take() {
                    stage.destroy();
                }

                // 空のステージを作成
                let mut stage = GuiStage::new();
                stage.set_player(self.seat);
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
                    self.seat = seat; // このメッセージはEventNewより先に受信
                }
                ServerMessage::Log => todo!(),
            }
        }

        if let Some(stage) = &mut self.stage
            && let Some(act) = stage.handle_gui_events()
        {
            self.tx
                .lock()
                .unwrap()
                .send(ClientMessage::Action(act))
                .unwrap();
        }

        // 使用されなかったbevyのMessageは次のフレームには持ち越されない
    }
}
