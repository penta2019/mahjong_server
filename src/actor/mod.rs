mod endpoint;
mod manual;
mod mjai;
mod nop;
mod null;
mod random;
mod tiitoitsu;

// use crate::controller::{Actor, Config, Listener};
use std::fmt;

use crate::listener::Listener;
use crate::model::*;
use crate::util::variant::*;

use crate::error;

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

trait ActorBuilder {
    fn get_default_config(&self) -> Config;
    fn create(&self, config: Config) -> Box<dyn Actor>;
}

pub fn create_actor(exp: &str) -> Box<dyn Actor> {
    let builders: Vec<Box<dyn ActorBuilder>> = vec![
        Box::new(null::NullBuilder {}),
        Box::new(nop::NopBuilder {}),
        Box::new(endpoint::EndpointBuilder {}),
        Box::new(random::RandomDiscardBuilder {}),
        Box::new(manual::ManualBuilder {}),
        Box::new(mjai::MjaiEndpointBuilder {}),
        Box::new(tiitoitsu::TiitoitsuBotBuilder {}),
    ];

    let name: &str;
    let args: Vec<&str>;
    let paren_left = exp.find('(');
    let paren_right = exp.rfind(')');
    if let (Some(l), Some(r)) = (paren_left, paren_right) {
        if r < l {
            error!("invalid paren: {}", exp);
            std::process::exit(0);
        }

        args = exp[l + 1..r].split(',').collect();
        name = &exp[..l];
    } else {
        args = vec![];
        name = exp;
    }

    for b in &builders {
        let mut conf = b.get_default_config();
        if name == conf.name {
            if conf.args.len() < args.len() {
                error!(
                    "expected {} arguments for {}. but {} arguments are provided.",
                    conf.args.len(),
                    name,
                    args.len(),
                );
                std::process::exit(0);
            }

            for (i, &a) in args.iter().enumerate() {
                if !a.is_empty() {
                    conf.args[i].value = match parse_as(&conf.args[i].value, a) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("{}: {}", e, a);
                            std::process::exit(0);
                        }
                    };
                }
            }

            return b.create(conf);
        }
    }

    error!("unknown actor name: {}", name);
    std::process::exit(0);
}

fn parse_as(target: &Variant, value: &str) -> Result<Variant, String> {
    Ok(match target {
        Variant::Int(_) => Variant::Int(value.parse::<i32>().map_err(|e| e.to_string())?),
        Variant::Float(_) => Variant::Float(value.parse::<f32>().map_err(|e| e.to_string())?),
        Variant::Bool(_) => Variant::Bool(value.parse::<bool>().map_err(|e| e.to_string())?),
        Variant::String(_) => Variant::String(value.parse::<String>().map_err(|e| e.to_string())?),
    })
}
