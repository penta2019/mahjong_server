mod event_printer;
mod event_writer;
mod stage_controller;

use std::fmt;

use crate::model::{Event, Stage};

pub use event_printer::{StageDebugPrinter, StagePrinter, StageStepPrinter};
pub use event_writer::{EventWriter, TenhouEventWriter};
pub use stage_controller::StageController;

pub trait EventListener: Send {
    fn notify_event(&mut self, _stg: &Stage, _event: &Event) {}
}

impl fmt::Debug for dyn EventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EventListener")
    }
}
