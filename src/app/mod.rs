// mainから直接呼び出すアプリケーションの動作モード(E, C, R, Jなど)のモジュール

mod calculator;
mod engine;
mod replay;

pub use calculator::CalculatorApp;
pub use engine::EngineApp;
pub use replay::ReplayApp;
