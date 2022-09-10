use serde_json::{json, Value};

use crate::controller::Listener;
use crate::convert::tenhou::TenhouSerializer;
use crate::model::*;
use crate::util::common::*;

// [EventWriter]
#[derive(Debug)]
pub struct EventWriter {
    start_time: u64,
    kyoku_index: i32,
    record: Vec<Value>,
}

impl EventWriter {
    pub fn new() -> Self {
        Self {
            start_time: unixtime_now(),
            kyoku_index: 0,
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
                self.start_time = unixtime_now();
                self.kyoku_index = 0;
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
            write_to_file(
                &format!("data/{}/{:2}.json", self.start_time, self.kyoku_index),
                &serde_json::to_string_pretty(&json!(self.record)).unwrap(),
            );
            self.record.clear();
            self.kyoku_index += 1;
        }
    }
}

// [TenhouEventWriter]
#[derive(Debug)]
pub struct TenhouEventWriter {
    start_time: u64,
    kyoku_index: i32,
    serializer: TenhouSerializer,
}

impl TenhouEventWriter {
    pub fn new() -> Self {
        Self {
            start_time: unixtime_now(),
            kyoku_index: 0,
            serializer: TenhouSerializer::new(),
        }
    }
}

impl Listener for TenhouEventWriter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        let mut write = false;
        match event {
            Event::Begin(_) => {
                self.start_time = unixtime_now();
                self.kyoku_index = 0;
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
                    "data_tenhou/{}/{:2}.json",
                    self.start_time, self.kyoku_index
                ),
                &self.serializer.serialize(),
            );
            self.kyoku_index += 1;
        }
    }
}
