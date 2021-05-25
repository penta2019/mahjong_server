use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::operator::*;
use crate::util::stage_listener::*;

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

// MjaiStageListener ==========================================================

pub struct MjaiStageListener {}
impl MjaiStageListener {}

impl StageListener for MjaiStageListener {
    fn notify_op_roundnew(
        &mut self,
        _stage: &Stage,
        _round: usize,
        _hand: usize,
        _ben: usize,
        _riichi_sticks: usize,
        _doras: &Vec<Tile>,
        _scores: &[i32; SEAT],
        _player_hands: &[Vec<Tile>; SEAT],
    ) {
    }

    fn notify_op_dealtile(&mut self, _stage: &Stage, _seat: Seat, _tile: Option<Tile>) {}

    fn notify_op_discardtile(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _tile: Tile,
        _is_drawn: bool,
        _is_riichi: bool,
    ) {
    }

    fn notify_op_chiiponkan(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _meld_type: MeldType,
        _tiles: &Vec<Tile>,
        _froms: &Vec<Seat>,
    ) {
    }

    fn notify_op_ankankakan(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _meld_type: MeldType,
        _tile: Tile,
    ) {
    }

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {}

    fn notify_op_roundend_win(
        &mut self,
        _stage: &Stage,
        _ura_doras: &Vec<Tile>,
        _contexts: &Vec<(Seat, WinContext)>,
        _delta_scores: &[i32; SEAT],
    ) {
    }

    fn notify_op_roundend_draw(&mut self, _stage: &Stage, _draw_type: DrawType) {}

    fn notify_op_roundend_notile(
        &mut self,
        _stage: &Stage,
        _is_ready: &[bool; SEAT],
        _delta_scores: &[i32; SEAT],
    ) {
    }
}
