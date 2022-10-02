mod debug;
mod event_printer;
mod event_sender;
mod event_writer;

use std::fmt;

use crate::model::*;

pub use debug::Debug;
pub use event_printer::EventPrinter;
pub use event_sender::{EventSender, StageSender};
pub use event_writer::{EventWriter, TenhouEventWriter};

pub trait Listener: Send {
    fn notify_event(&mut self, _stg: &Stage, _event: &Event) {}
    // fn notify_actions(&mut self, _stg: &Stage, actions: &Vec<UserAction>) {}
}

impl fmt::Debug for dyn Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Listener")
    }
}
