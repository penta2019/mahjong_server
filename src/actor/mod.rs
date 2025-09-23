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

use std::any::Any;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

use crate::control::stage_controller::StageRef;
use crate::listener::Listener;
use crate::model::*;
use crate::util::misc::Res;
use crate::util::variant::*;

use crate::error;

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub args: Vec<Arg>,
}

pub type SelectedAction = Pin<Box<dyn Future<Output = Action>>>;

pub fn ready(act: Action) -> SelectedAction {
    Box::pin(std::future::ready(act))
}

// Actor trait
pub trait Actor: Listener + ActorClone + Send {
    // 局開始時の初期化処理
    // 卓情報の参照は_stageを構造体変数に保存しておいてselectが呼ばれた時に
    // StageRef.lockで読み込み専用のRwReadGuardを獲得して行う.
    // StageRef.lockは別スレッドから実行する場合は稀に失敗することがある.
    // アクションを選択し終わった後は確実にRwReadGuardをdrop(=ロックの開放)すること.
    fn init(&mut self, _stage: StageRef, _seat: Seat) {}

    // アクションの選択
    // actsの中から任意のアクションを選択して返すFutureを返す.
    // tenpaisは聴牌可能な時に捨て牌と和了牌の組み合わせを示す.
    // Rust1.75でasync traitが実装されたがtraitオブジェクトと一緒には使えない.
    fn select(&mut self, acts: &[Action], tenpais: &[Tenpai]) -> SelectedAction;

    // アクションの選択の失効通知
    // Actorがアクションの選択を行う前にアクションの選択自体が不可能になった場合に呼ばれる.
    // これは優先度の高いアクション(ロンなど)が他家によって行われた場合やタイムアウトした場合などに起こる.
    // メインスレッドの処理はStageRefのすべてのロックが開放されるまでブロッキングするため
    // このメソッドが呼ばれた際はStageRefのロックをすぐに開放することが望ましい.
    fn expire(&mut self) {}

    // Actorの詳細表示用
    fn get_config(&self) -> &Config;

    // プレイヤー名
    fn get_name(&self) -> &str {
        &self.get_config().name
    }

    // ダウンキャストが必要な場合に実装 (Gui用)
    fn try_as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
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

            for (i, &a) in args.iter().enumerate() {
                if !a.is_empty() {
                    conf.args[i].value = match parse_as(&conf.args[i].value, a) {
                        Ok(v) => v,
                        Err(err) => {
                            error!("{}: {}", err, a);
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
