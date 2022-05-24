use super::*;

pub struct EndpointBuilder;

impl ActorBuilder for EndpointBuilder {
    fn get_default_config(&self) -> Config {
        Config {
            name: "Endpoint".to_string(),
            args: vec![Arg::string("addr", "127.0.0.1:11611")],
        }
    }

    fn create(&self, config: Config) -> Box<dyn Actor> {
        Box::new(Endpoint::from_config(config))
    }
}

#[derive(Clone)]
pub struct Endpoint {
    config: Config,
}

impl Endpoint {
    pub fn from_config(config: Config) -> Self {
        let args = &config.args;
        let addr = args[0].value.as_string();
        Endpoint { config: config }
    }
}

impl Actor for Endpoint {
    fn select_action(&mut self, _stg: &Stage, _acts: &Vec<Action>, _repeat: i32) -> Option<Action> {
        panic!();
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl Listener for Endpoint {}
