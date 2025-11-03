use std::path::{Path, PathBuf};

use mahjong_core::{
    control::{actor::Actor, listener::Listener, stage_controller::StageController},
    error,
    model::*,
    serde_json,
    util::misc::*,
};

use crate::listener::{Debug, EventPrinter};

#[derive(Debug)]
pub struct ReplayApp {
    file_path: String,
    skip: String,
    debug: bool,
    // names: [String; SEAT], // actor names
}

impl ReplayApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            file_path: String::new(),
            skip: String::new(),
            debug: false,
            // names: [
            //     String::new(),
            //     String::new(),
            //     String::new(),
            //     String::new(),
            // ],
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-f" => app.file_path = next_value(&mut it, s),
                "-s" => app.skip = next_value(&mut it, s),
                "-d" => app.debug = true,
                opt => {
                    error!("unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.file_path.is_empty() {
            error!("file(-f) not specified");
            exit(0);
        }

        app
    }

    pub fn run(&mut self) {
        let mut listeners: Vec<Box<dyn Listener>> = vec![];

        listeners.push(Box::new(EventPrinter::new()));
        if self.debug {
            listeners.push(Box::new(Debug::new()));
        }

        // パスがディレクトリならそのディレクトリ内のすべてのjsonファイルを読み込む
        let path = Path::new(&self.file_path);
        let paths: Vec<std::path::PathBuf> = if path.is_dir() {
            get_paths(path)
                .unwrap_or_else(error_exit)
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
        let mut skips: Vec<usize> = if self.skip.is_empty() {
            vec![]
        } else {
            self.skip
                .split(',')
                .map(|s| s.parse().unwrap_or_else(error_exit))
                .collect()
        };
        while skips.len() < 3 {
            skips.push(0);
        }
        let rkh = (skips[0], skips[1], skips[2]);

        let mut game = Replay::new(listeners);
        for p in paths {
            println!("source file: {:?}\n", p);
            let contents = std::fs::read_to_string(p).unwrap_or_else(error_exit);
            let record: Vec<Event> = serde_json::from_str(&contents).unwrap();

            if let Event::New(ev) = &record[0]
                && (ev.round, ev.dealer, ev.honba) < rkh
            {
                continue;
            }

            game.run(record);
        }
    }
}

#[derive(Debug)]
struct Replay {
    ctrl: StageController,
}

impl Replay {
    fn new(listeners: Vec<Box<dyn Listener>>) -> Self {
        let nop = crate::actor::create_actor("Nop");
        let nops: [Box<dyn Actor>; SEAT] = [
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
        ];

        Self {
            ctrl: StageController::new(nops, listeners),
        }
    }

    fn run(&mut self, events: Vec<Event>) {
        let mut cursor = 0;
        while cursor < events.len() {
            self.ctrl.handle_event(&events[cursor]);
            cursor += 1
        }
    }
}
