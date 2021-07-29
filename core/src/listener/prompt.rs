use crate::controller::Listener;
use crate::model::*;
use crate::util::common::prompt;

pub struct Prompt {}

impl Prompt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for Prompt {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        if let Event::DealTile(_) = event {
        } else {
            prompt();
        }
    }
}
