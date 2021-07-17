use std::path::{Path, PathBuf};

use serde_json::json;

use crate::actor::{create_actor, Actor};
use crate::controller::{StageController, StageStepPrinter};
use crate::model::*;
use crate::util::common::*;
use crate::util::ws_server::*;

#[derive(Debug)]
pub struct ReplayApp {
    file_path: String,
    skip: String,
    gui_port: u32,
}

impl ReplayApp {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            file_path: String::new(),
            skip: String::new(),
            gui_port: super::GUI_PORT,
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-f" => app.file_path = next_value(&mut it, "-f"),
                "-s" => app.skip = next_value(&mut it, "-s"),
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port"),
                opt => {
                    println!("Unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.file_path == "" {
            println!("file(-f) not specified");
            exit(0);
        }

        app
    }

    pub fn run(&mut self) {
        let nop = create_actor("Nop");
        let actors: [Box<dyn Actor>; SEAT] = [
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
            nop.clone_box(),
        ];
        let mut ctrl = StageController::new(actors, vec![Box::new(StageStepPrinter {})]);
        let send_recv = create_ws_server(self.gui_port);

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

        for p in paths {
            let contents = std::fs::read_to_string(&p).unwrap_or_else(print_and_exit);
            let record: Vec<Event> = serde_json::from_str(&contents).unwrap();

            if let Event::RoundNew(e) = &record[0] {
                if (e.round, e.kyoku, e.honba) < rkh {
                    continue;
                }
            }

            for r in &record {
                ctrl.handle_event(&r);
                if let Some((s, _)) = send_recv.lock().unwrap().as_ref() {
                    let msg = json!({
                        "type": "stage",
                        "data": &json!(ctrl.get_stage()),
                    });
                    s.send(msg.to_string()).ok();
                }
                prompt();
            }
        }
    }
}
