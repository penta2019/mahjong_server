use crate::listener::Listener;
use crate::model::*;

pub type EventTx = std::sync::mpsc::Sender<Event>;
pub type EventRx = std::sync::mpsc::Receiver<Event>;

pub struct EventChannel {
    event_tx: EventTx,
}

impl EventChannel {
    pub fn new(event_tx: EventTx) -> Self {
        Self { event_tx }
    }
}

impl Listener for EventChannel {
    fn notify_event(&mut self, _stg: &Stage, event: &Event) {
        self.event_tx.send(event.clone()).ok();
    }
}
