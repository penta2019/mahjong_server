use super::*;

use SelectedAction::*;

pub struct NopBuilder;

impl ActorBuilder for NopBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Nop".to_string(),
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
    fn select_action(&mut self, _acts: &[Action], _tenpais: &[Tenpai]) -> SelectedAction {
        Sync(Action::nop())
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Nop {}
