use std::fmt;

use crate::hand::evaluate::WinContext;
use crate::model::*;

// StageListener (Observer Pattern)
pub trait StageListener: Send {
    fn notify_op_game_start(&mut self, _stage: &Stage) {}

    fn notify_op_roundnew(
        &mut self,
        _stage: &Stage,
        _round: usize,
        _kyoku: usize,
        _honba: usize,
        _kyoutaku: usize,
        _doras: &Vec<Tile>,
        _scores: &[Score; SEAT],
        _player_hands: &[Vec<Tile>; SEAT],
    ) {
    }

    fn notify_op_dealtile(&mut self, _stage: &Stage, _seat: Seat, _tile: Tile) {}

    fn notify_op_discardtile(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _tile: Tile,
        _is_drawn: bool,
        _is_riichi: bool,
    ) {
    }

    fn notify_op_chiponkan(
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

    fn notify_op_dora(&mut self, _stage: &Stage, _tile: Tile) {}

    fn notify_op_kita(&mut self, _stage: &Stage, _seat: Seat, _is_drawn: bool) {}

    fn notify_op_roundend_win(
        &mut self,
        _stage: &Stage,
        _ura_doras: &Vec<Tile>,
        _contexts: &Vec<(Seat, [Point; SEAT], WinContext)>,
    ) {
    }

    fn notify_op_roundend_draw(&mut self, _stage: &Stage, _draw_type: DrawType) {}

    fn notify_op_roundend_notile(
        &mut self,
        _stage: &Stage,
        _is_ready: &[bool; SEAT],
        _points: &[Point; SEAT],
    ) {
    }

    fn notify_op_game_over(&mut self, _stage: &Stage) {}
}

impl fmt::Debug for dyn StageListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StageListener")
    }
}
