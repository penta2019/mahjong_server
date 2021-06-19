use super::stage_controller::StageListener;
use crate::model::*;
use crate::util::common::vec_to_string;

// [StagePrinter]
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
    fn notify_action(&mut self, stg: &Stage, act: &Action) {
        match act {
            Action::GameStart(_) => {}
            Action::RoundNew(_) => {
                println!("[ROUNDNEW]");
                println!("{}", stg);
            }
            Action::DealTile(_) => {}
            Action::DiscardTile(_) => {}
            Action::Meld(_) => {}
            Action::Kita(_) => {}
            Action::Dora(_) => {}
            Action::RoundEndWin(a) => {
                println!("[ROUNDEND]");
                println!("ura_dora: {}", vec_to_string(&a.ura_doras));
                println!("{:?}", a.contexts);
                let mut deltas = [0; SEAT];
                for ctx in &a.contexts {
                    for s in 0..SEAT {
                        deltas[s] += ctx.1[s];
                    }
                }

                self.print_score_change(&stg, &deltas);
                println!("{}", stg);
            }
            Action::RoundEndDraw(a) => {
                println!("[ROUNDEND DRAW]");
                println!("{:?}", a.draw_type);
                println!("{}", stg);
            }
            Action::RoundEndNoTile(a) => {
                println!("[ROUNDEND NOTILE]");
                println!("is_tenpai: {:?}", &a.tenpais);
                self.print_score_change(&stg, &a.points);
                println!("{}", stg);
            }
            Action::GameOver(_) => {}
        }
    }
}

// [StageDebugPrinter]
pub struct StageDebugPrinter {}

impl StageDebugPrinter {}

impl StageListener for StageDebugPrinter {
    fn notify_action(&mut self, stg: &Stage, act: &Action) {
        println!("step: {}", stg.step);
        println!("{}", serde_json::to_string(act).unwrap());
    }
}

// [StageStepPrinter]
