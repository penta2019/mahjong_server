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
    // 局開始時の初期化処理
    fn init(&mut self, _seat: Seat) {}

    // 可能なアクションの選択
    // 処理を非同期に行う必要がある場合,Noneを返すことで100ms以内に同じ選択に対して再度この関数が呼び出される.
    // この時,呼び出されるたびにrepeatに1加算される.
    // 各々の選択に対して初回の呼び出しでは retry=0 である.
    fn select_action(&mut self, stg: &Stage, acts: &Vec<Action>, retry: i32) -> Option<Action>;

    // Actorの詳細表示用
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
        write!(f, "{}({})", conf.name, arg_str)
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
