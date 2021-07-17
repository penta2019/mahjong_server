use super::*;

pub struct NullBuilder;

impl ActorBuilder for NullBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Null".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Null::from_config(config))
    }
}

#[derive(Clone)]
pub struct Null {
    config: Config,
}

impl Null {
    pub fn from_config(config: Config) -> Self {
        Null { config: config }
    }
}

impl Actor for Null {
    fn select_action(&mut self, _stage: &Stage, _seat: Seat, _operatons: &Vec<Action>) -> Action {
        panic!();
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl EventListener for Null {}
