use serde_json::{json, Value};

use crate::controller::Listener;
use crate::convert::tenhou::{TenhouLog, TenhouSerializer};
use crate::model::*;
use crate::util::common::*;

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
            start_time: unixtime_now(),
            round_index: 0,
            record: vec![],
        }
    }
}

impl Listener for EventWriter {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        let mut write = false;
        match event {
            Event::GameStart(_) => {
                self.record.clear();
                self.start_time = unixtime_now();
                self.round_index = 0;
            }
            Event::RoundNew(_) => {
                self.record.clear();
            }
            Event::RoundEndWin(_) | Event::RoundEndDraw(_) | Event::RoundEndNoTile(_) => {
                write = true;
            }
            Event::GameOver(_) => {}
            _ => {}
        }

        self.record.push(json!(event));
        if write {
            write_to_file(
                &format!("data/{}/{:2}.json", self.start_time, self.round_index),
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
    pub fn new(log: TenhouLog) -> Self {
        Self {
            start_time: unixtime_now(),
            round_index: 0,
            serializer: TenhouSerializer::new(log),
        }
    }
}

impl Listener for TenhouEventWriter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        let mut write = false;
        match event {
            Event::GameStart(_) => {
                self.start_time = unixtime_now();
                self.round_index = 0;
            }
            Event::RoundEndWin(_) | Event::RoundEndDraw(_) | Event::RoundEndNoTile(_) => {
                write = true;
            }
            Event::GameOver(_) => {}
            _ => {}
        }

        self.serializer.push_event(stg, event);
        if write {
            write_to_file(
                &format!("data/{}/{:2}.json", self.start_time, self.round_index),
                &self.serializer.serialize(),
            );
            self.round_index += 1;
        }
    }
}
