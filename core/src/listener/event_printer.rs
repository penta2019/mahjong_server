use crate::controller::Listener;
use crate::model::*;
use crate::util::common::vec_to_string;

// [StagePrinter]
pub struct StagePrinter {}

impl StagePrinter {
    pub fn new() -> Self {
        Self {}
    }

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

impl Listener for StagePrinter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        match event {
            Event::GameStart(_) => {
                println!("[GameStart]");
            }
            Event::RoundNew(_) => {
                println!("[RoundNew]");
                println!("{}", stg);
            }
            Event::DealTile(_) => {}
            Event::DiscardTile(_) => {}
            Event::Meld(_) => {}
            Event::Kita(_) => {}
            Event::Dora(_) => {}
            Event::RoundEndWin(e) => {
                println!("[RoundEndWin]");
                println!("ura_dora: {}", vec_to_string(&e.ura_doras));
                println!("{:?}", e.contexts);
                let mut deltas = [0; SEAT];
                for ctx in &e.contexts {
                    for s in 0..SEAT {
                        deltas[s] += ctx.1[s];
                    }
                }

                self.print_score_change(&stg, &deltas);
                println!("{}", stg);
            }
            Event::RoundEndDraw(e) => {
                println!("[RoundEndDraw]");
                println!("{:?}", e.draw_type);
                println!("{}", stg);
            }
            Event::RoundEndNoTile(e) => {
                println!("[RoundEndNoTile]");
                println!("is_tenpai: {:?}", &e.tenpais);
                self.print_score_change(&stg, &e.points);
                println!("{}", stg);
            }
            Event::GameOver(_) => {
                println!("[GameOver]");
            }
        }
    }
}

// [StageStepPrinter]
pub struct StageStepPrinter {}

impl StageStepPrinter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for StageStepPrinter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        println!("step: {}", stg.step);
        match event {
            Event::GameStart(_) => {
                println!("[GameStart]");
            }
            Event::RoundNew(_) => {
                println!("[RoundNew]");
                println!("{}", stg);
            }
            Event::DealTile(e) => {
                println!("[DealTile]");
                println!("{}", stg.players[e.seat]);
            }
            Event::DiscardTile(e) => {
                println!("[DiscardTile]");
                println!("{}", stg.players[e.seat]);
            }
            Event::Meld(e) => {
                println!("[Meld]");
                println!("{}", stg.players[e.seat]);
            }
            Event::Kita(e) => {
                println!("[Kita]");
                println!("{}", stg.players[e.seat]);
            }
            Event::Dora(_) => {
                println!("[Dora]");
                println!("{:?}", stg.doras);
            }
            Event::RoundEndWin(_) => {
                println!("[RoundEndWin]");
                println!("{}", stg);
            }
            Event::RoundEndDraw(_) => {
                println!("[RoundEndDraw]");
                println!("{}", stg);
            }
            Event::RoundEndNoTile(_) => {
                println!("[RoundEndNoTile]");
                println!("{}", stg);
            }
            Event::GameOver(_) => {
                println!("[GameOver]");
                println!("{}", stg);
            }
        }
    }
}

// [StageDebugPrinter]
pub struct StageDebugPrinter {}

impl StageDebugPrinter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for StageDebugPrinter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        println!("step: {}", stg.step);
        println!("{}", serde_json::to_string(event).unwrap());
    }
}
