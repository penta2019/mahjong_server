// Actorトレイトを継承して打牌の判断を行う構造体のモジュール(AI,プレイヤー,botなど)
mod endpoint;
mod manual;
mod mjai;
mod nop;
mod null;
mod random;
mod tiitoitsu;

#[cfg(feature = "gui")]
pub mod gui;

use std::{any::Any, future::Future, pin::Pin};

use mahjong_core::{
    control::{
        actor::{ActionFuture, Actor, Config, ready},
        stage_controller::StageRef,
    },
    error,
    listener::Listener,
    model::*,
    util::misc::Res,
    util::variant::*,
};

trait ActorBuilder {
    fn get_default_config(&self) -> Config;
    fn create(&self, config: Config) -> Box<dyn Actor>;
}

pub fn create_actor(exp: &str) -> Box<dyn Actor> {
    let builders: Vec<Box<dyn ActorBuilder>> = vec![
        Box::new(null::NullBuilder),
        Box::new(nop::NopBuilder),
        Box::new(endpoint::EndpointBuilder),
        Box::new(random::RandomDiscardBuilder),
        Box::new(manual::ManualBuilder),
        Box::new(mjai::MjaiEndpointBuilder),
        Box::new(tiitoitsu::TiitoitsuBotBuilder),
        #[cfg(feature = "gui")]
        Box::new(gui::GuiBuilder),
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

            for (i, &arg) in args.iter().enumerate() {
                if !arg.is_empty() {
                    conf.args[i].value = match parse_as(&conf.args[i].value, arg) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("{}: {}", err, arg);
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

fn parse_as(target: &Variant, value: &str) -> Res<Variant> {
    Ok(match target {
        Variant::Int(_) => Variant::Int(value.parse::<i32>()?),
        Variant::Float(_) => Variant::Float(value.parse::<f32>()?),
        Variant::Bool(_) => Variant::Bool(value.parse::<bool>()?),
        Variant::String(_) => Variant::String(value.parse::<String>()?),
    })
}
