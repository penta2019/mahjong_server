use crate::controller::stage_controller::StageController;
use crate::model::*;
use crate::operator::nop::Nop;
use crate::operator::Operator;

#[derive(Debug)]
pub struct App {
    ctrl: StageController,
    file_name: String,
}

impl App {
    pub fn new(args: Vec<String>) -> Self {
        if args.len() < 1 {
            println!("[Error] file path not specified");
        }

        let nop = Box::new(Nop::new());
        let operators: [Box<dyn Operator>; SEAT] =
            [nop.clone(), nop.clone(), nop.clone(), nop.clone()];
        App {
            ctrl: StageController::new(operators, vec![]),
            file_name: args[0].clone(),
        }
    }

    pub fn run(&mut self) {}
}
