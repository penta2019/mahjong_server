use std::io::Write;

use crate::controller::stage_controller::StageListener;
use crate::model::*;
use crate::util::common::*;
use crate::util::tenhou::{TenhouLog, TenhouSerializer};

#[derive(Debug)]
pub struct EventWriter {
    start_time: u64,
    round_index: i32,
    serializer: TenhouSerializer,
}

impl EventWriter {
    pub fn new(log: TenhouLog) -> Self {
        Self {
            start_time: unixtime_now(),
            round_index: 0,
            serializer: TenhouSerializer::new(log),
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
        let data = self.serializer.serialize();
        write!(f, "{}", data).unwrap();
    }
}

impl StageListener for EventWriter {
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
            self.write_to_file();
            self.round_index += 1;
        }
    }
}
