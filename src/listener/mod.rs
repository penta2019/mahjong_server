// Listernerトレイトを継承する構造体のモジュール
mod debug;
mod event_printer;
mod event_sender;
mod event_writer;
mod message_channel;

use std::fmt;

use crate::model::*;

pub use self::{
    debug::Debug,
    event_printer::EventPrinter,
    event_sender::{EventSender, StageSender},
    event_writer::{EventWriter, TenhouEventWriter},
    message_channel::MessageChannel,
};

pub trait Listener: Send {
    fn notify_event(&mut self, _stg: &Stage, _event: &Event) {}
    // fn notify_actions(&mut self, _stg: &Stage, actions: &[UserAction]) {}
}

impl fmt::Debug for dyn Listener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Listener")
    }
}
