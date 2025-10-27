use std::{any::Any, fmt, future::Future, pin::Pin};

use crate::{control::stage_controller::StageRef, listener::Listener, model::*, util::variant::*};

#[derive(Debug, Clone)]
pub struct Config {
    pub name: String,
    pub args: Vec<Arg>,
}

pub type ActionFuture = Pin<Box<dyn Future<Output = Action>>>;

pub fn ready(act: Action) -> ActionFuture {
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
    fn select(&mut self, acts: &[Action], tenpais: &[Tenpai]) -> ActionFuture;

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
            .map(|arg| format!("{}={}", arg.name, arg.value))
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
