mod manual;
mod mjai;
mod nop;
mod null;
mod random;
mod tiitoitsu;

use std::fmt;

use crate::controller::EventListener;
use crate::model::*;
use crate::util::variant::*;

// Actor trait
pub trait Actor: EventListener + ActorClone + Send {
    fn set_seat(&mut self, _: Seat) {}
    fn select_action(&mut self, stage: &Stage, seat: Seat, operatons: &Vec<Action>) -> Action;
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

#[derive(Clone)]
pub struct Config {
    name: String,
    args: Vec<Arg>,
}

trait ActorBuilder {
    fn get_default_config(&self) -> Config;
    fn create(&self, config: Config) -> Box<dyn Actor>;
}

pub fn create_actor(exp: &str) -> Box<dyn Actor> {
    let builders: Vec<Box<dyn ActorBuilder>> = vec![
        Box::new(null::NullBuilder {}),
        Box::new(nop::NopBuilder {}),
        Box::new(random::RandomDiscardBuilder {}),
        Box::new(manual::ManualBuilder {}),
        Box::new(mjai::MjaiEndpointBuilder {}),
        Box::new(tiitoitsu::TiitoitsuBotBuilder {}),
    ];

    let name: &str;
    let args: Vec<&str>;
    let paren_left = exp.find('(');
    let paren_right = exp.rfind(')');
    if paren_left.is_some() && paren_right.is_some() {
        let l = paren_left.unwrap();
        let r = paren_right.unwrap();
        if r < l {
            println!("[Error] Invalid parent: {}", exp);
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
                println!(
                    "expected {} arguments for {}. but {} arguments are provided.",
                    conf.args.len(),
                    name,
                    args.len(),
                );
                std::process::exit(0);
            }

            for (i, &a) in args.iter().enumerate() {
                if a != "" {
                    conf.args[i].value = match parse_as(&conf.args[i].value, a) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("[Error] {}: \"{}\"", e, a);
                            std::process::exit(0);
                        }
                    };
                }
            }

            return b.create(conf);
        }
    }

    println!("Unknown actor name: {}", name);
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
