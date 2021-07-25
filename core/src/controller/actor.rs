use std::fmt;

use super::*;
use crate::model::*;
use crate::util::variant::*;

#[derive(Clone)]
pub struct Config {
    pub name: String,
    pub args: Vec<Arg>,
}

// Actor trait
pub trait Actor: Listener + ActorClone + Send {
    fn init(&mut self, _seat: Seat) {}
    fn select_action(&mut self, stage: &Stage, seat: Seat, actions: &Vec<Action>) -> Action;
    fn get_config(&self) -> &Config;
}

impl fmt::Debug for dyn Actor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let conf = self.get_config();
        let arg_str = conf
            .args
            .iter()
            .map(|a| format!("{}={}", a.name, a.value))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "Actor: {}({})", conf.name, arg_str)
    }
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait ActorClone {
    fn clone_box(&self) -> Box<dyn Actor>;
}

impl<T> ActorClone for T
where
    T: 'static + Actor + Clone,
{
    fn clone_box(&self) -> Box<dyn Actor> {
        Box::new(self.clone())
    }
}
