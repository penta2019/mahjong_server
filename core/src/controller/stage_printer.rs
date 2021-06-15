use super::stage_listener::StageListener;
use crate::hand::evaluate::WinContext;
use crate::model::*;
use crate::util::common::vec_to_string;

// StagePrinter
pub struct StagePrinter {}

impl StagePrinter {
    fn print_score_change(&self, stage: &Stage, delta_scores: &[Score; SEAT]) {
        for s in 0..SEAT {
            let delta = delta_scores[s];
            let new = stage.players[s].score;
            let old = new - delta;
            println!("Player {}: {} -> {} ({:+})", s, old, new, delta);
        }
        println!();
    }
}

impl StageListener for StagePrinter {
    fn notify_op_roundnew(
        &mut self,
        stage: &Stage,
        _round: usize,
        _kyoku: usize,
        _honba: usize,
        _kyoutaku: usize,
        _doras: &Vec<Tile>,
        _scores: &[Score; SEAT],
        _player_hands: &[Vec<Tile>; SEAT],
    ) {
        println!("[ROUNDNEW]");
        println!("{}", stage);
    }

    fn notify_op_roundend_win(
        &mut self,
        stage: &Stage,
        ura_doras: &Vec<Tile>,
        contexts: &Vec<(Seat, [Point; SEAT], WinContext)>,
    ) {
        println!("[ROUNDEND]");
        println!("ura_dora: {}", vec_to_string(ura_doras));
        println!("{:?}", contexts);
        let mut deltas = [0; SEAT];
        for ctx in contexts {
            for s in 0..SEAT {
                deltas[s] += ctx.1[s];
            }
        }

        self.print_score_change(&stage, &deltas);
        println!("{}", stage);
    }

    fn notify_op_roundend_draw(&mut self, stage: &Stage, draw_type: DrawType) {
        println!("[ROUNDEND DRAW]");
        println!("{:?}", draw_type);
        println!("{}", stage);
    }

    fn notify_op_roundend_notile(
        &mut self,
        stage: &Stage,
        is_ready: &[bool; SEAT],
        points: &[Point; SEAT],
    ) {
        println!("[ROUNDEND NOTILE]");
        println!("is_ready: {:?}", is_ready);
        self.print_score_change(&stage, &points);
        println!("{}", stage);
    }
}
