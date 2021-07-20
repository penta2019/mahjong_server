mod manual;
mod mjai;
mod nop;
mod null;
mod random;
mod tiitoitsu;

use crate::controller::{Actor, Config, Listener};
use crate::model::*;
use crate::util::variant::*;

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
