use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::operator::*;
use crate::util::stage_listener::StageListener;

use PlayerOperation::*;

// MjaiEndpoint ===============================================================

#[derive(Clone)]
pub struct MjaiEndpoint {}

impl MjaiEndpoint {
    pub fn new(addr: &str) -> Self {
        // thread::spawn(move || {
        //     let listener = TcpListener::bind(addr).unwrap();
        // })
        Self {}
    }
}

impl Operator for MjaiEndpoint {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        ops: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        Nop
    }

    fn debug_string(&self) -> String {
        "MjaiEndpoint".to_string()
    }
}

impl StageListener for MjaiEndpoint {}
