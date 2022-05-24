use std::sync::mpsc;
use std::thread;

use serde_json::{json, Value};

use crate::controller::Listener;
use crate::model::*;
use crate::util::common::sleep_ms;
use crate::util::server::Server;

// [EventSender]
#[derive(Debug)]
pub struct EventSender {
    sender: mpsc::Sender<Value>,
}

impl EventSender {
    pub fn new(mut server: Server) -> Self {
        let (s, r) = mpsc::channel::<Value>();
        thread::spawn(move || {
            let mut msgs: Vec<Value> = vec![];
            let mut cursor = 0;
            loop {
                while let Ok(msg) = r.try_recv() {
                    if msg["type"].as_str().unwrap() == "New" {
                        msgs.clear();
                        cursor = 0;
                    }
                    msgs.push(msg);
                }

                if server.is_new() {
                    cursor = 0;
                }

                if server.is_connected() {
                    while cursor < msgs.len() {
                        server.send(msgs[cursor].to_string());
                        cursor += 1;
                    }
                }

                sleep_ms(100);
            }
        });
        Self { sender: s }
    }
}

impl Listener for EventSender {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        self.sender.send(json!(event)).ok();
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
