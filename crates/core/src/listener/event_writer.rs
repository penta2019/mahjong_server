use serde_json::{Value, json};

use crate::{
    control::listener::Listener, convert::tenhou::TenhouSerializer, model::*, util::misc::*,
};

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

impl Default for EventWriter {
    fn default() -> Self {
        Self::new()
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
            write_to_file(
                &format!(
                    "local/paifu/{}/{:02}.json",
                    self.start_time, self.round_index
                ),
                &serde_json::to_string_pretty(&json!(self.record)).unwrap(),
            )
            .ok();
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

impl Default for TenhouEventWriter {
    fn default() -> Self {
        Self::new()
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
                    "local/paifu_tenhou/{}/{:02}.json",
                    self.start_time, self.round_index
                ),
                &self.serializer.serialize(),
            )
            .ok();
            self.round_index += 1;
        }
    }
}
