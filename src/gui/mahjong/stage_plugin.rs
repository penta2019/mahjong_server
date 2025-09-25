use std::sync::{
    Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::input::{ButtonState, mouse::MouseButtonInput};

use super::*;
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
            .add_systems(Update, process_event);
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
            stage.set_target_tile(entity);
        }
    }
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
