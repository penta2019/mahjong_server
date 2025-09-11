use crate::error;

#[derive(Debug)]
pub struct ClientApp {
    args: Vec<String>,
}

impl ClientApp {
    pub fn new(args: Vec<String>) -> Self {
        if !cfg!(feature = "gui") {
            error!("`gui` feature is required at compile time");
            std::process::exit(1);
        }
        Self { args }
    }

    pub fn run(&mut self) {
        assert!(self.args.is_empty());
        #[cfg(feature = "gui")]
        {
            let (_, rx) = std::sync::mpsc::channel(); // TODO
            crate::gui::Gui::new().run(rx);
        }
    }
}
