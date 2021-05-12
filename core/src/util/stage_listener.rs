use std::fmt;

use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::common::vec_to_string;

// StageListener (Observer Pattern)
pub trait StageListener: Send {
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

impl fmt::Debug for dyn StageListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StageListener")
    }
}

// StageConsolePrinter
pub struct StageConsolePrinter {}

impl StageConsolePrinter {
    fn print_score_change(&self, stage: &Stage, delta_scores: &[i32; SEAT]) {
        for s in 0..SEAT {
            let delta = delta_scores[s];
            let new = stage.players[s].score;
            let old = new - delta;
            println!("Player {}: {} -> {} ({:+})", s, old, new, delta);
        }
        println!();
    }
}

impl StageListener for StageConsolePrinter {
    fn notify_op_roundnew(
        &mut self,
        stage: &Stage,
        _round: usize,
        _hand: usize,
        _ben: usize,
        _riichi_sticks: usize,
        _doras: &Vec<Tile>,
        _scores: &[i32; SEAT],
        _player_hands: &[Vec<Tile>; SEAT],
    ) {
        println!("[ROUNDNEW]");
        stage.print();
    }

    fn notify_op_roundend_win(
        &mut self,
        stage: &Stage,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, WinContext)>,
        delta_scores: &[i32; SEAT],
    ) {
        println!("[ROUNDEND]");
        println!("ura_dora: {}", vec_to_string(ura_doras));
        println!("{:?}", contexts);
        self.print_score_change(&stage, &delta_scores);
        stage.print();
    }

    fn notify_op_roundend_draw(&mut self, stage: &Stage, draw_type: DrawType) {
        println!("[ROUNDEND DRAW]");
        println!("{:?}", draw_type);
        println!();
        stage.print();
    }

    fn notify_op_roundend_notile(
        &mut self,
        stage: &Stage,
        is_ready: &[bool; SEAT],
        delta_scores: &[i32; SEAT],
    ) {
        println!("[ROUNDEND NOTILE]");
        println!("is_ready: {:?}", is_ready);
        self.print_score_change(&stage, &delta_scores);
        stage.print();
    }
}
