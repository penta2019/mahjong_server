use crate::{listener::Listener, model::*, util::misc::vec_to_string};

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
            Deal(ev) => {
                println!("Deal {}", ev.tile);
                println!("{}", pl);
            }
            Discard(ev) => {
                println!(
                    "Discard {} {}",
                    ev.tile,
                    if ev.is_riichi { "riichi" } else { "" }
                );
                println!("{}", pl);
            }
            Meld(_) => {
                println!("Meld");
                println!("{}", pl);
            }
            Nukidora(_) => {
                println!("Nukidora");
                println!("{}", pl);
            }
            Dora(ev) => {
                println!("Dora {}", ev.tile);
            }
            Win(ev) => {
                println!("Win");
                println!("doras: {}", vec_to_string(&ev.doras));
                println!("ura_doras: {}", vec_to_string(&ev.ura_doras));
                println!("{:?}", ev.contexts);
                self.print_score_change(stg, &ev.delta_scores);
                println!("{}", stg);
            }
            Draw(ev) => {
                println!("Draw");
                println!("{}", stg);
                self.print_score_change(stg, &ev.delta_scores);
            }
            End(_) => {
                println!("End");
            }
        }
        println!();
    }
}
