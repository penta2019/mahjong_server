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
        Self { config }
    }
}

impl Actor for Null {
    fn select(&mut self, _acts: &[Action], _tenpais: &[Tenpai]) -> ActionFuture {
        panic!();
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Null {}
