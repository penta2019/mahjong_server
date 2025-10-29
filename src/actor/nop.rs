use super::*;

pub struct NopBuilder;

impl ActorBuilder for NopBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Nop".into(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Nop::from_config(config))
    }
}

#[derive(Clone)]
pub struct Nop {
    config: Config,
}

impl Nop {
    pub fn from_config(config: Config) -> Self {
        Self { config }
    }
}

impl Actor for Nop {
    fn select(&mut self, _acts: &[Action], _tenpais: &[Tenpai]) -> ActionFuture {
        ready(Action::nop())
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Nop {}
