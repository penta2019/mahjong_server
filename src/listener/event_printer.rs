use crate::controller::Listener;
use crate::model::*;
use crate::util::common::vec_to_string;

// [StagePrinter]
#[derive(Debug)]
pub struct EventPrinter {}

impl EventPrinter {
    pub fn new() -> Self {
        Self {}
    }

    fn print_score_change(&self, stg: &Stage, d_scores: &[Point; SEAT]) {
        for s in 0..SEAT {
            let delta = d_scores[s];
            let new = stg.players[s].score;
            let old = new - delta;
            println!("player {}: {} -> {} ({:+})", s, old, new, delta);
        }
        println!();
    }
}

impl Listener for EventPrinter {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        use Event::*;
        print!("(step:{}) ", stg.step);
        let pl = &stg.players[stg.turn];
        match event {
            Begin(_) => {
                println!("Begin");
            }
            New(_) => {
                println!("New");
                println!("{}", stg);
            }
            Deal(e) => {
                println!("Deal {}", e.tile);
                println!("{}", pl);
            }
            Discard(e) => {
                println!(
                    "Discard {} {}",
                    e.tile,
                    if e.is_riichi { "riichi" } else { "" }
                );
                println!("{}", pl);
            }
            Meld(_) => {
                println!("Meld");
                println!("{}", pl);
            }
            Kita(_) => {
                println!("Kita");
                println!("{}", pl);
            }
            Dora(e) => {
                println!("Dora {}", e.tile);
            }
            Win(e) => {
                println!("Win");
                println!("ura_dora: {}", vec_to_string(&e.ura_doras));
                println!("{:?}", e.contexts);
                let mut deltas = [0; SEAT];
                for ctx in &e.contexts {
                    for s in 0..SEAT {
                        deltas[s] += ctx.1[s];
                    }
                }
                self.print_score_change(stg, &deltas);
                println!("{}", stg);
            }
            Draw(e) => {
                println!("Draw");
                println!("{}", stg);
                self.print_score_change(stg, &e.delta_scores);
            }
            End(_) => {
                println!("End");
            }
        }
        println!();
    }
}
