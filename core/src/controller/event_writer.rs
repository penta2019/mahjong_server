use std::io::Write;

use serde_json::{json, Value};

use crate::controller::stage_controller::StageListener;
use crate::model::*;
use crate::util::common::*;

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

    fn write_to_file(&mut self) {
        let file_name = format!(
            "data/{}/{:02}.json",
            self.start_time.to_string(),
            self.round_index
        );
        let path = std::path::Path::new(&file_name);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let mut f = std::fs::File::create(path).unwrap();
        let data = serde_json::to_string_pretty(&json!(self.record)).unwrap();
        write!(f, "{}", data).unwrap();
    }
}

impl StageListener for EventWriter {
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
            self.write_to_file();
            self.record.clear();
            self.round_index += 1;
        }
    }
}
