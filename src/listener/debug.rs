use crate::controller::Listener;
use crate::model::*;
use crate::util::common::prompt;

pub struct Debug {}

impl Debug {
    pub fn new() -> Self {
        Self {}
    }
}

impl Listener for Debug {
    fn notify_event(&mut self, stg: &Stage, event: &Event) {
        loop {
            print!("debug");
            let input = prompt();
            let args: Vec<&str> = input.split_whitespace().collect();
            // println!("{:?}", args);
            if args.len() == 0 {
                break;
            }

            match args[0] {
                "p" => {
                    println!("{}", stg);
                }
                "e" => {
                    println!("{:?}", event);
                }
                "s" => {
                    println!("{:?}", stg);
                }
                _ => {
                    println!("unknown command: {}\n", args[0]);
                    println!("usage");
                    println!("[Enter]: next step");
                    println!("p: print stage info");
                    println!("e: debug-print current Event");
                    println!("s: debug-print Stage")
                }
            }
            println!();
        }
    }
}
