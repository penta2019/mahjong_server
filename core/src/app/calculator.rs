use crate::{error, warn};

#[derive(Debug)]
pub struct CalculatorApp {
    exp: String,
}

impl CalculatorApp {
    pub fn new(args: Vec<String>) -> Self {
        let mut app = Self {
            exp: "".to_string(),
        };

        let mut it = args.iter();
        while let Some(s) = it.next() {
            match s.as_str() {
                exp => {
                    if exp.starts_with("-") {
                        error!("unknown option: {}", exp);
                        std::process::exit(0);
                    }
                }
            }
        }

        app
    }

    pub fn run(&mut self) {}
}

#[derive(Debug)]
struct Calculator {}

impl Calculator {
    fn new() -> Self {
        Self {}
    }
}
