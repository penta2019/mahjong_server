use std::sync::{Arc, Mutex};
use std::thread;

use serde_json::{json, Value};

use crate::etc::connection::{Connection, Message};
use crate::etc::misc::sleep;
use crate::listener::Listener;
use crate::model::*;

#[derive(Debug, Default)]
struct SharedData {
    send_request: bool,
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
                d.send_request = false;
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
            sleep(0.01);
        });

        Self { data: arc0 }
    }
}

impl Listener for EventSender {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        {
            let mut d = self.data.lock().unwrap();
            if let Event::New(_) = event {
                d.msgs.clear();
                d.cursor = 0;
            }
            d.msgs.push(json!(event));
            d.send_request = true;
        }

        while self.data.lock().unwrap().send_request {
            sleep(0.01); // pushしたデータが処理されるまで待機
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
