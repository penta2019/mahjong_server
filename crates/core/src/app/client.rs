#[derive(Debug)]
pub struct ClientApp {
    args: Vec<String>,
}

impl ClientApp {
    pub fn new(args: Vec<String>) -> Self {
        #[cfg(not(feature = "gui"))]
        {
            crate::error!("`gui` feature is required at compile time");
            std::process::exit(1);
        }
        Self { args }
    }

    pub fn run(&mut self) {
        assert!(self.args.is_empty());
        #[cfg(feature = "gui")]
        {
            let (_, rx) = std::sync::mpsc::channel(); // TODO
            let (tx, _) = std::sync::mpsc::channel(); // TODO
            crate::gui::run(tx, rx);
        }
    }
}
