use serde_json::{json, Value};

use crate::controller::Listener;
use crate::model::*;
use crate::util::connection::{Connection, Message};

// [EventSender]
#[derive(Debug)]
pub struct EventSender {
    conn: Box<dyn Connection>,
    msgs: Vec<Value>,
}

impl EventSender {
    pub fn new(conn: Box<dyn Connection>) -> Self {
        Self {
            conn,
            msgs: vec![],
        }
    }
}

impl Listener for EventSender {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        if let Event::New(_) = event {
            self.msgs.clear();
        }
        self.msgs.push(json!(event));

        match self.conn.recv() {
            Message::Open => {
                for m in &self.msgs {
                    self.conn.send(&m.to_string());
                }
            }
            Message::Close => {}
            _ => {
                if let Some(m) = self.msgs.iter().last() {
                    self.conn.send(&m.to_string());
                }
            }
        }
    }
}

// [StageSender]
#[derive(Debug)]
pub struct StageSender {
    conn: Box<dyn Connection>,
}

impl StageSender {
    pub fn new(conn: Box<dyn Connection>) -> Self {
        Self { conn }
    }
}

impl Listener for StageSender {
    fn notify_event(&mut self, stg: &Stage, _event: &Event) {
        match self.conn.recv() {
            Message::Close => {}
            _ => self.conn.send(&json!(stg).to_string()),
        }
    }
}
