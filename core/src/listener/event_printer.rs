use crate::controller::Listener;
use crate::model::*;
use crate::util::common::vec_to_string;

// [StagePrinter]
#[derive(Debug)]
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
            println!("player {}: {} -> {} ({:+})", s, old, new, delta);
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
#[derive(Debug)]
pub struct StageStepPrinter {}

impl StageStepPrinter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for StageStepPrinter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        use Event::*;
        let ev_str = format!("{:?}", event);
        print!("[{}] ", ev_str.split('(').next().unwrap().to_string());
        println!("(step: {})", stg.step);
        match event {
            GameStart(_) => {}
            DealTile(_) | DiscardTile(_) | Meld(_) | Kita(_) => {
                println!("{}", stg.players[stg.turn]);
            }
            Dora(_) => {
                println!("{:?}", stg.doras);
            }
            RoundNew(_) | RoundEndWin(_) | RoundEndDraw(_) | GameOver(_) => {
                println!("{}", stg);
            }
        }
        println!();
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
