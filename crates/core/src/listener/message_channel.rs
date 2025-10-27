use std::sync::mpsc::Sender;

use crate::{listener::Listener, model::*};

pub struct MessageChannel {
    event_tx: Sender<ServerMessage>,
}

impl MessageChannel {
    pub fn new(event_tx: Sender<ServerMessage>) -> Self {
        Self { event_tx }
    }
}

impl Listener for MessageChannel {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        self.event_tx
            .send(ServerMessage::Event(Box::new(event.clone())))
            .ok();
    }
}
