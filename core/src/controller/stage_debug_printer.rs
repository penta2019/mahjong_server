use super::stage_controller::StageListener;
use crate::model::*;

// StageDebugPrinter
pub struct StageDebugPrinter {}

impl StageDebugPrinter {}

impl StageListener for StageDebugPrinter {
    fn notify_action(&mut self, stg: &Stage, act: &Action) {
        println!("step: {}", stg.step);
        println!("{}", serde_json::to_string(act).unwrap());
    }
}
