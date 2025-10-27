// mainから直接呼び出すアプリケーションの動作モード(E, C, Rなど)のモジュール
mod calculator;
mod client;
mod engine;
mod replay;

pub use self::{
    calculator::CalculatorApp, client::ClientApp, engine::EngineApp, replay::ReplayApp,
};
