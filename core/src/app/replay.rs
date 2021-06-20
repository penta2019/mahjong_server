use serde_json::json;

use crate::controller::stage_controller::StageController;
use crate::controller::stage_printer::StageStepPrinter;
use crate::model::*;
use crate::operator::nop::Nop;
use crate::operator::Operator;
use crate::util::common::*;
use crate::util::ws_server::*;

#[derive(Debug)]
pub struct App {
    file_name: String,
    gui_port: u32,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        use std::process::exit;

        let mut app = Self {
            file_name: String::new(),
            gui_port: super::GUI_PORT,
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                "-f" => app.file_name = next_value(&mut it, "-f: input file name missing"),
                "-gui-port" => app.gui_port = next_value(&mut it, "-gui-port: port number missing"),
                opt => {
                    println!("Unknown option: {}", opt);
                    exit(0);
                }
            }
        }

        if app.file_name == "" {
            println!("file(-f) not specified");
            exit(0);
        }

        app
    }

    pub fn run(&mut self) {
        use std::process::exit;

        let nop = Box::new(Nop::new());
        let operators: [Box<dyn Operator>; SEAT] =
            [nop.clone(), nop.clone(), nop.clone(), nop.clone()];
        let mut ctrl = StageController::new(operators, vec![Box::new(StageStepPrinter {})]);
        let send_recv = create_ws_server(self.gui_port);

        let contents = match std::fs::read_to_string(&self.file_name) {
            Ok(c) => c,
            Err(err) => {
                println!("[Error] {}", err);
                exit(0);
            }
        };

        let record: Vec<Action> = serde_json::from_str(&contents).unwrap();
        for r in &record {
            ctrl.handle_action(&r);

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
