use serde_json::{json, Value};

use crate::controller::Listener;
use crate::model::*;
use crate::util::server::Server;

// [EventSender]
#[derive(Debug)]
pub struct EventSender {
    server: Server,
    record: Vec<Value>,
}

impl EventSender {
    pub fn new(server: Server) -> Self {
        Self {
            server: server,
            record: vec![],
        }
    }
}

impl Listener for EventSender {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        match event {
            Event::GameStart(_) => {
                self.record.clear();
            }
            Event::RoundNew(_) => {
                self.record.clear();
            }
            Event::RoundEndWin(_) | Event::RoundEndDraw(_) | Event::RoundEndNoTile(_) => {}
            Event::GameOver(_) => {}
            _ => {}
        }

        self.record.push(json!(event));

        if self.server.is_connected() {
            let msg = self.record[self.record.len() - 1].to_string();
            self.server.send(msg);
        }
    }
}

// [StageSender]
#[derive(Debug)]
pub struct StageSender {
    server: Server,
}

impl StageSender {
    pub fn new(server: Server) -> Self {
        Self { server: server }
    }
}

impl Listener for StageSender {
    fn notify_event(&mut self, stg: &Stage, _event: &Event) {
        if self.server.is_connected() {
            let value = json!({
                "type": "stage",
                "data": stg,
            });
            self.server.send(value.to_string());
        }
    }
}
