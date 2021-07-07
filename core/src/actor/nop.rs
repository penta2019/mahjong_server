use super::*;

pub struct NopBuilder;

impl OperatorBuilder for NopBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Nop".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Operator> {
        Box::new(Nop::from_config(config))
    }
}

#[derive(Clone)]
pub struct Nop {
    config: Config,
}

impl Nop {
    pub fn new() -> Self {
        Self::from_config((NopBuilder {}).get_default_config())
    }

    pub fn from_config(config: Config) -> Self {
        Nop { config: config }
    }
}

impl Operator for Nop {
    fn select_action(&mut self, _stage: &Stage, _seat: Seat, _operatons: &Vec<Action>) -> Action {
        Action::nop()
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for Nop {}
