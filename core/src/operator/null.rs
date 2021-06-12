use super::*;

pub struct NullBuilder;

impl OperatorBuilder for NullBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Null".to_string(),
            args: vec![],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Operator> {
        Box::new(Null::from_config(config))
    }
}

#[derive(Clone)]
pub struct Null {
    config: Config,
}

impl Null {
    pub fn new() -> Self {
        Self::from_config((NullBuilder {}).get_default_config())
    }

    pub fn from_config(config: Config) -> Self {
        Null { config: config }
    }
}

impl Operator for Null {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation {
        panic!();
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl StageListener for Null {}
