use serde_json::json;

use crate::controller::Listener;
use crate::model::*;
use crate::util::ws_server::{create_ws_server, SendRecv};

use crate::info;

pub struct GuiServer {
    server: SendRecv,
}

impl GuiServer {
    pub fn new(port: u32) -> Self {
        Self {
            server: create_ws_server(port),
        }
    }
}

impl Listener for GuiServer {
    fn notify_event(&mut self, stg: &Stage, _event: &Event) {
        if let Some((s, r)) = self.server.lock().unwrap().as_ref() {
            // 送られてきたメッセージをすべて表示
            loop {
                match r.try_recv() {
                    Ok(msg) => {
                        info!("ws message: {}", msg);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            // stageの状態をjsonにエンコードして送信
            let value = json!({
                "type": "stage",
                "data": stg,
            });
            s.send(value.to_string()).ok();
        }
    }
}
