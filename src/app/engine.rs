use std::sync::mpsc;

use mahjong_core::{
    control::{actor::Actor, engine::MahjongEngine, listener::Listener},
    error, info,
    listener::*,
    model::*,
    util::misc::*,
};
use rand::prelude::*;

use crate::actor::*;

// [App]
#[derive(Debug)]
pub struct EngineApp {
    seed: u64,
    rule: Rule,
    pause: f64,
    n_game: u32,
    n_thread: u32,
    view: bool,
    write: bool,
    write_tenhou: bool,
    debug: bool,
    quiet: bool,
    names: [String; SEAT], // actor names
}

impl EngineApp {
    pub fn new(args: Vec<String>) -> Self {
        let mut app = Self {
            seed: 0,
            rule: Rule {
                round: 1,
                is_sanma: false,
                initial_score: 25000,
                settlement_score: 30000,
                red5: 1,
                bust: true,
            },
            pause: 0.0,
            n_game: 0,
            n_thread: 16,
            view: false,
            write: false,
            write_tenhou: false,
            debug: false,
            quiet: false,
            names: ["Nop".into(), "Nop".into(), "Nop".into(), "Nop".into()],
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-s" => app.seed = next_value(&mut it, s),
                "-r-round" => app.rule.round = next_value(&mut it, s),
                "-r-init" => app.rule.initial_score = next_value(&mut it, s),
                "-r-settle" => app.rule.settlement_score = next_value(&mut it, s),
                "-r-red5" => app.rule.red5 = next_value(&mut it, s),
                "-r-bust" => app.rule.bust = next_value(&mut it, s),
                "-p" => app.pause = next_value(&mut it, s),
                "-g" => app.n_game = next_value(&mut it, s),
                "-t" => app.n_thread = next_value(&mut it, s),
                "-v" => app.view = true,
                "-w" => app.write = true,
                "-w-tenhou" => app.write_tenhou = true,
                "-d" => app.debug = true,
                "-q" => app.quiet = true,
                "-0" => app.names[0] = next_value(&mut it, s),
                "-1" => app.names[1] = next_value(&mut it, s),
                "-2" => app.names[2] = next_value(&mut it, s),
                "-3" => app.names[3] = next_value(&mut it, s),
                opt => {
                    error!("unknown option: {}", opt);
                    std::process::exit(0);
                }
            }
        }

        if app.seed == 0 {
            app.seed = unixtime_now() as u64;
            info!(
                "Random seed is not specified. Unix timestamp '{}' is used as seed.",
                app.seed
            );
        }

        assert!(app.rule.bust); // TODO: 飛びなしルール実装

        app
    }

    pub fn run(self) {
        println!("seed: {}", self.seed);

        let actors = [
            create_actor(&self.names[0]),
            create_actor(&self.names[1]),
            create_actor(&self.names[2]),
            create_actor(&self.names[3]),
        ];
        for s in 0..SEAT {
            println!("actor{}: {:?}", s, actors[s]);
        }
        println!();

        let start = std::time::Instant::now();
        if self.n_game == 0 {
            self.run_single_game(actors);
        } else {
            self.run_multiple_game(actors);
        }
        println!(
            "total elapsed time: {:8.3}sec",
            start.elapsed().as_nanos() as f32 / 1000000000.0
        );
    }

    fn run_single_game(self, mut actors: [Box<dyn Actor>; 4]) {
        let mut listeners: Vec<Box<dyn Listener>> = vec![];
        if !self.quiet {
            listeners.push(Box::new(EventPrinter::new()));
        }
        if self.write {
            listeners.push(Box::new(EventWriter::new()));
        }
        if self.write_tenhou {
            listeners.push(Box::new(mahjong_core::listener::TenhouEventWriter::new()));
        }
        if self.debug {
            listeners.push(Box::new(Debug::new()));
        }

        #[cfg(feature = "gui")]
        {
            let mut gui_txrx = None;
            for actor in &mut actors {
                if let Some(any) = actor.try_as_any_mut()
                    && let Some(actor_gui) = any.downcast_mut::<crate::actor::gui::Gui>()
                {
                    if gui_txrx.is_some() {
                        error!("Multiple `Gui` cannot exist simultaneously.");
                        std::process::exit(1);
                    }
                    gui_txrx = actor_gui.take_client_txrx();
                }
            }

            if self.view || gui_txrx.is_some() {
                let (tx, rx) = if let Some((tx, rx)) = gui_txrx {
                    (tx, rx)
                } else {
                    let (tx, _server_rx) = mpsc::channel(); // upstream
                    let (server_tx, rx) = mpsc::channel(); // downstream
                    listeners.push(Box::new(MessageChannel::new(server_tx)));
                    (tx, rx)
                };

                std::thread::spawn(move || {
                    let mut game = MahjongEngine::new(
                        self.seed,
                        self.rule.clone(),
                        self.pause,
                        actors,
                        listeners,
                    );
                    game.run();
                });
                mahjong_gui::run(tx, rx);
                return;
            }
        }

        #[cfg(not(feature = "gui"))]
        if self.view {
            error!("view mode `-v` requires `gui` feature at compile time");
            std::process::exit(1);
        }

        let mut game =
            MahjongEngine::new(self.seed, self.rule.clone(), self.pause, actors, listeners);
        game.run();
    }

    fn run_multiple_game(self, actors: [Box<dyn Actor>; 4]) {
        use std::{thread, time};

        let mut n_game = 0;
        let mut n_thread = 0;
        let mut n_game_end = 0;
        let mut rng: rand::rngs::StdRng = rand::SeedableRng::seed_from_u64(self.seed);
        let (tx, rx) = mpsc::channel();
        let mut sum_delta_scores = [0; SEAT];
        let mut sum_ranks = [0; SEAT];
        loop {
            if n_game < self.n_game && n_thread < self.n_thread {
                n_game += 1;
                n_thread += 1;

                let seed = rng.next_u64();
                let mut shuffle_table = [0, 1, 2, 3];
                shuffle_table.shuffle(&mut rng);
                let null = create_actor("Null");
                let mut shuffled_actors: [Box<dyn Actor>; SEAT] = [
                    null.clone_box(),
                    null.clone_box(),
                    null.clone_box(),
                    null.clone_box(),
                ];
                for s in 0..SEAT {
                    shuffled_actors[s] = actors[shuffle_table[s]].clone_box();
                }

                let rule = self.rule.clone();
                let pause = self.pause;
                let tx2 = tx.clone();
                thread::spawn(move || {
                    let start = time::Instant::now();
                    let mut game = MahjongEngine::new(seed, rule, pause, shuffled_actors, vec![]);
                    game.run();
                    tx2.send((shuffle_table, game, start.elapsed())).unwrap();
                });
            }

            loop {
                if let Ok((shuffle, game, elapsed)) = rx.try_recv() {
                    let ms = elapsed.as_nanos() / 1000000;
                    print!("{:5},{:4}ms,{:20}", n_game_end, ms, game.get_seed());
                    for s in 0..SEAT {
                        let pl = &game.get_stage().players[s];
                        let (score, rank) = (pl.score, pl.rank + 1);
                        let i = shuffle[s];
                        sum_delta_scores[i] += score - self.rule.initial_score;
                        sum_ranks[i] += rank;
                        print!(", ac{}:{:5}({})", i, score, rank);
                    }
                    println!();

                    n_thread -= 1;
                    n_game_end += 1;
                }
                if n_thread < self.n_thread {
                    break;
                }
                sleep(0.01);
            }

            if n_thread == 0 && n_game == self.n_game {
                for i in 0..SEAT {
                    println!(
                        "ac{} avg_rank: {:.2}, avg_delta_score: {:6}",
                        i,
                        sum_ranks[i] as f32 / n_game as f32,
                        sum_delta_scores[i] / n_game as i32,
                    );
                }
                break;
            }
        }
    }
}
