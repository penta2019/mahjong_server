// mainから直接呼び出すアプリケーションの動作モード(E, C, R, Jなど)のモジュール

mod calculator;
mod engine;
mod mahjongsoul;
mod replay;

const MSC_PORT: u32 = 52000;

pub use calculator::CalculatorApp;
pub use engine::EngineApp;
pub use mahjongsoul::MahjongsoulApp;
pub use replay::ReplayApp;
