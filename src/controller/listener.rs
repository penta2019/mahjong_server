use std::fmt;

use crate::model::*;

pub trait Listener: Send {
    fn notify_event(&mut self, _stg: &Stage, _event: &Event) {}
}

impl fmt::Debug for dyn Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Listener")
    }
}
