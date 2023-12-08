use serde_json::{json, Value};

use crate::convert::tenhou::TenhouSerializer;
use crate::listener::Listener;
use crate::model::*;
use crate::util::misc::*;

// [EventWriter]
#[derive(Debug)]
pub struct EventWriter {
    start_time: u64,
    round_index: i32,
    record: Vec<Value>,
}

impl EventWriter {
    pub fn new() -> Self {
        Self {
            start_time: unixtime_now() as u64,
            round_index: 0,
            record: vec![],
        }
    }
}

impl Listener for EventWriter {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        let mut write = false;
        match event {
            Event::Begin(_) => {
                self.record.clear();
                self.start_time = unixtime_now() as u64;
                self.round_index = 0;
            }
            Event::New(_) => {
                self.record.clear();
            }
            Event::Win(_) | Event::Draw(_) => {
                write = true;
            }
            Event::End(_) => {}
            _ => {}
        }

        self.record.push(json!(event));
        if write {
            let _ = write_to_file(
                &format!("data/{}/{:02}.json", self.start_time, self.round_index),
                &serde_json::to_string_pretty(&json!(self.record)).unwrap(),
            );
            self.record.clear();
            self.round_index += 1;
        }
    }
}

// [TenhouEventWriter]
#[derive(Debug)]
pub struct TenhouEventWriter {
    start_time: u64,
    round_index: i32,
    serializer: TenhouSerializer,
}

impl TenhouEventWriter {
    pub fn new() -> Self {
        Self {
            start_time: unixtime_now() as u64,
            round_index: 0,
            serializer: TenhouSerializer::new(),
        }
    }
}

impl Listener for TenhouEventWriter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        let mut write = false;
        match event {
            Event::Begin(_) => {
                self.start_time = unixtime_now() as u64;
                self.round_index = 0;
            }
            Event::Win(_) | Event::Draw(_) => {
                write = true;
            }
            Event::End(_) => {}
            _ => {}
        }

        self.serializer.push_event(stg, event);
        if write {
            write_to_file(
                &format!(
                    "data_tenhou/{}/{:02}.json",
                    self.start_time, self.round_index
                ),
                &self.serializer.serialize(),
            )
            .ok();
            self.round_index += 1;
        }
    }
}
