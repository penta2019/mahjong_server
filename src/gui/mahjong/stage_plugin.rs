use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender},
};

use super::*;
use crate::{
    gui::mahjong::player::PossibleActions,
    model::{Event as MjEvent, *},
};

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
            .add_systems(Update, process_event);
    }
}

#[derive(Resource, Debug)]
pub struct StageResource {
    stage: Option<GuiStage>,
    seat: Seat,
    tx: Arc<Mutex<Tx>>,
    rx: Mutex<Rx>,
}

impl StageResource {
    fn new(tx: Tx, rx: Rx) -> Self {
        Self {
            stage: None,
            seat: 0,
            tx: Arc::new(Mutex::new(tx)),
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
                    param().commands.entity(stage.entity()).despawn();
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
                    // GuiTileのentityが追加される前にget()が呼び出され失敗する
                    break;
                }
                ServerMessage::Action {
                    id,
                    actions,
                    tenpais,
                } => {
                    self.stage
                        .as_mut()
                        .unwrap()
                        .handle_actions(PossibleActions::new(
                            id,
                            actions,
                            tenpais,
                            self.tx.clone(),
                        ));
                }
                ServerMessage::Info { seat } => {
                    self.seat = seat; // このメッセージはEventNewより先に受信
                }
            }
        }

        if let Some(stage) = &mut self.stage {
            stage.handle_gui_events();
        }
        // 使用されなかったEventを全て破棄
        param().drain_events();
    }
}

fn process_event(mut param: StageParam, mut stage_res: ResMut<StageResource>) {
    with_param(&mut param, || {
        stage_res.update();
    });
}
