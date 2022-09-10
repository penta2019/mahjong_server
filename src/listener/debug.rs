use crate::controller::Listener;
use crate::model::*;
use crate::util::common::prompt;

pub struct Debug {}

impl Debug {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for Debug {
    fn notify_event(&mut self, _stg: &Stage, _event: &Event) {
        prompt();
    }
}
