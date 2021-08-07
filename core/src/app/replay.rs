use std::path::{Path, PathBuf};

use crate::actor::create_actor;
use crate::controller::*;
use crate::listener::{Prompt, StageSender, StageStepPrinter};
use crate::model::*;
use crate::util::common::*;
use crate::util::server::Server;

use crate::error;

#[derive(Debug)]
pub struct ReplayApp {
    file_path: String,
    skip: String,
    gui_port: u32,
    debug: bool,
    names: [String; SEAT], // actor names
}

impl ReplayApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            file_path: String::new(),
            skip: String::new(),
            gui_port: super::GUI_PORT,
            debug: false,
            names: [
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ],
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-f" => app.file_path = next_value(&mut it, "-f"),
                "-s" => app.skip = next_value(&mut it, "-s"),
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port"),
                "-d" => app.debug = true,
                "-0" => app.names[0] = next_value(&mut it, "-0"),
                "-1" => app.names[1] = next_value(&mut it, "-1"),
                "-2" => app.names[2] = next_value(&mut it, "-2"),
                "-3" => app.names[3] = next_value(&mut it, "-3"),
                opt => {
                    error!("unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.file_path == "" {
            error!("file(-f) not specified");
            exit(0);
        }

        app
    }

    pub fn run(&mut self) {
        let nop = create_actor("Nop");
        let mut actors: [Box<dyn Actor>; SEAT] = [
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
        ];
        let mut enabled_actors = [false; SEAT];
        for i in 0..SEAT {
            let n = &self.names[i];
            if n != "" {
                actors[i] = create_actor(n);
                enabled_actors[i] = true;
            }
        }
        for s in 0..SEAT {
            println!("actor{}: {:?}", s, actors[s]);
        }
        println!();

        let mut listeners: Vec<Box<dyn Listener>> = vec![];
        listeners.push(Box::new(StageStepPrinter::new()));
        let server = Server::new_ws_server(&format!("localhost:{}", self.gui_port));
        listeners.push(Box::new(StageSender::new(server)));
        listeners.push(Box::new(Prompt::new()));

        // パスがディレクトリならそのディレクトリ内のすべてのjsonファイルを読み込む
        let path = Path::new(&self.file_path);
        let paths: Vec<std::path::PathBuf> = if path.is_dir() {
            get_paths(path)
                .unwrap_or_else(print_and_exit)
                .into_iter()
                .filter(|p| match p.extension() {
                    Some(ext) => ext == "json",
                    None => false,
                })
                .collect()
        } else {
            let mut buf = PathBuf::new();
            buf.push(&self.file_path);
            vec![buf]
        };

        // スキップ位置の情報をパース
        let mut skips: Vec<usize> = if self.skip == "" {
            vec![]
        } else {
            self.skip
                .split(',')
                .map(|s| s.parse().unwrap_or_else(print_and_exit))
                .collect()
        };
        while skips.len() < 3 {
            skips.push(0);
        }
        let rkh = (skips[0], skips[1], skips[2]);

        let mut game = Replay::new(actors, enabled_actors, listeners);
        for p in paths {
            let contents = std::fs::read_to_string(p).unwrap_or_else(print_and_exit);
            let record: Vec<Event> = serde_json::from_str(&contents).unwrap();

            if let Event::RoundNew(e) = &record[0] {
                if (e.round, e.kyoku, e.honba) < rkh {
                    continue;
                }
            }

            game.run(record);
        }
    }
}

#[derive(Debug)]
struct Replay {
    enabled_actors: [bool; SEAT],
    ctrl: StageController,
    melding: Option<Action>,
    is_round_end: bool,
    events: Vec<Event>,
    cursor: usize, // eventsのindex
}

impl Replay {
    fn new(
        mut actors: [Box<dyn Actor>; SEAT],
        enabled_actors: [bool; SEAT],
        listeners: Vec<Box<dyn Listener>>,
    ) -> Self {
        for s in 0..SEAT {
            actors[s].init(s);
        }

        Self {
            enabled_actors: enabled_actors,
            ctrl: StageController::new(actors, listeners),
            melding: None,
            is_round_end: false,
            events: vec![],
            cursor: 0,
        }
    }

    fn run(&mut self, events: Vec<Event>) {
        self.events = events;
        self.cursor = 0;
        self.is_round_end = false;

        self.do_round_new();
        loop {
            self.check_kan_dora(); // 暗槓の槓ドラ(不要だが念の為)
            self.do_turn_operation();
            if self.is_round_end {
                break;
            }

            self.check_kan_dora(); // 明槓,加槓の槓ドラ
            self.do_call_operation();
            if self.is_round_end {
                break;
            }

            self.check_kan_dora(); // 暗槓の槓ドラ
            self.do_deal_tile();
            if self.is_round_end {
                break;
            }
        }
        self.do_round_end();
    }

    fn get_stage(&self) -> &Stage {
        self.ctrl.get_stage()
    }

    fn get_event(&self) -> &Event {
        &self.events[self.cursor]
    }

    fn handle_event(&mut self) {
        self.ctrl.handle_event(&self.events[self.cursor]);
        self.cursor += 1;
    }

    fn check_kan_dora(&mut self) {
        let e = self.get_event();
        match e {
            Event::Dora(_) => {
                self.handle_event();
            }
            _ => {}
        }
    }

    fn do_round_new(&mut self) {
        let e = self.get_event();
        match e {
            Event::RoundNew(_) => {
                self.handle_event();
            }
            _ => panic!(),
        }
    }

    fn do_turn_operation(&mut self) {
        let stg = self.get_stage();
        let turn = stg.turn;
        let acts = calc_possible_turn_actions(stg, &self.melding);
        let act = self.ctrl.select_action(turn, &acts);

        let e = self.get_event();
        let act2 = match e {
            Event::DiscardTile(e) => {
                if e.is_riichi {
                    if e.is_drawn {
                        Action::riichi(e.tile) // TODO
                    } else {
                        Action::riichi(e.tile)
                    }
                } else {
                    if e.is_drawn {
                        Action::nop()
                    } else {
                        Action::discard(e.tile)
                    }
                }
            }
            Event::Meld(e) => {
                let a = match e.meld_type {
                    MeldType::Ankan => Action::ankan(e.consumed.clone()),
                    MeldType::Kakan => Action::kakan(e.consumed.clone()[0]),
                    _ => panic!(),
                };
                self.melding = Some(a.clone());
                a
            }
            Event::RoundEndWin(_) => {
                self.is_round_end = true;
                Action::tsumo()
            }
            Event::RoundEndDraw(_) => Action::kyushukyuhai(),
            Event::Kita(_) => Action::kita(),
            _ => panic!(),
        };

        println!("selected: {:?}, actual: {:?}", act, act2);
        self.handle_event();
    }

    fn do_call_operation(&mut self) {
        let e = self.get_event();
        match e {
            Event::RoundEndWin(_) => {
                self.is_round_end = true;
            }
            Event::Meld(e) => {
                // self.melding =
                match e.meld_type {
                    MeldType::Chi => {}
                    MeldType::Pon => {}
                    MeldType::Minkan => {}
                    _ => panic!(),
                }
            }
            _ => return,
        }

        self.handle_event();
    }

    fn do_deal_tile(&mut self) {
        let e = self.get_event();

        match e {
            Event::DealTile(_) => {}
            Event::RoundEndNoTile(_) => {
                self.is_round_end = true;
            }
            Event::Meld(_) => return, // Chi, pon
            _ => panic!(),
        }
        self.handle_event();
    }

    fn do_round_end(&mut self) {}
}
