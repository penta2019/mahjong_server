// ControlParam(= param())に依存するStage UI関連のモジュールはここに配置
mod discard;
mod hand;
mod meld;
mod player;
mod stage;
mod tile;

use std::{
    f32::consts::{FRAC_PI_2, PI},
    sync::Mutex,
};

use bevy::prelude::*;

use super::control_param::param;

use super::{Rx, Tx};
use crate::{
    gui::{move_animation::MoveAnimation, util::reparent_tranform},
    model::{Event as MjEvent, *},
};

use self::{
    discard::GuiDiscard,
    hand::{GuiHand, IsDrawn},
    meld::GuiMeld,
    player::{GuiPlayer, HandMode},
    stage::GuiStage,
    tile::GuiTile,
};

trait HasEntity {
    fn entity(&self) -> Entity;
}

#[derive(Resource, Debug)]
pub struct MahjonGuiControl {
    stage: Option<GuiStage>,
    seat: Seat,
    tx: Mutex<Tx>,
    rx: Mutex<Rx>,
}

impl MahjonGuiControl {
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
                    // GuiTileのentityが追加される前にget()が呼び出され失敗する
                    break;
                }
                ServerMessage::Action(possible_actions) => {
                    self.stage
                        .as_mut()
                        .unwrap()
                        .handle_actions(possible_actions);
                }
                ServerMessage::Info { seat } => {
                    self.seat = seat; // このメッセージはEventNewより先に受信
                }
            }
        }

        if let Some(stage) = &mut self.stage
            && let Some(action) = stage.handle_gui_events()
        {
            self.tx
                .lock()
                .unwrap()
                .send(ClientMessage::Action(action))
                .unwrap();
        }
        // 使用されなかったEventを全て破棄
        param().drain_events();
    }
}
