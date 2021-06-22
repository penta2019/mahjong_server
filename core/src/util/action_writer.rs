use std::io::Write;

use serde_json::json;

use crate::model::*;
use crate::util::common::*;

#[derive(Debug)]
pub struct ActionWriter {
    start_time: u64,
    round_index: i32,
    record: Vec<Action>,
}

impl ActionWriter {
    pub fn new() -> Self {
        Self {
            start_time: unixtime_now(),
            round_index: 0,
            record: vec![],
        }
    }

    pub fn push_action(&mut self, act: Action) {
        let mut write = false;
        match act {
            Action::GameStart(_) => {
                self.record.clear();
                self.start_time = unixtime_now();
                self.round_index = 0;
            }
            Action::RoundNew(_) => {
                self.record.clear();
            }
            Action::RoundEndWin(_) | Action::RoundEndDraw(_) | Action::RoundEndNoTile(_) => {
                write = true;
            }
            Action::GameOver(_) => {}
            _ => {}
        }

        self.record.push(act);
        if write {
            self.write_to_file();
            self.record.clear();
            self.round_index += 1;
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
