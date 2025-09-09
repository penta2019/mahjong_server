// mainから直接呼び出すアプリケーションの動作モード(E, C, Rなど)のモジュール

mod calculator;
mod client;
mod engine;
mod replay;

pub use calculator::CalculatorApp;
pub use client::ClientApp;
pub use engine::EngineApp;
pub use replay::ReplayApp;
