use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use crate::controller::Listener;
use crate::model::*;
use crate::util::common::sleep_ms;
use crate::util::connection::{Connection, Message};

#[derive(Debug, Default)]
struct SharedData {
    msgs: Vec<Value>,
    cursor: usize,
}

// [EventSender]
#[derive(Debug)]
pub struct EventSender {
    data: Arc<Mutex<SharedData>>,
}

impl EventSender {
    pub fn new(mut conn: Box<dyn Connection>) -> Self {
        let arc0 = Arc::new(Mutex::new(SharedData::default()));
        let arc1 = arc0.clone();

        thread::spawn(move || loop {
            loop {
                let mut d = arc1.lock().unwrap();
                match conn.recv() {
                    Message::Open => d.cursor = 0,
                    Message::Text(_) => {}
                    Message::NoMessage => {
                        while d.cursor < d.msgs.len() {
                            conn.send(&d.msgs[d.cursor].to_string());
                            d.cursor += 1;
                        }
                        break;
                    }
                    Message::Close => {}
                    Message::NoConnection => break,
                }
            }
            sleep_ms(100);
        });

        Self { data: arc0 }
    }
}

impl Listener for EventSender {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        let mut d = self.data.lock().unwrap();
        if let Event::New(_) = event {
            d.msgs.clear();
            d.cursor = 0;
        }
        d.msgs.push(json!(event));
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