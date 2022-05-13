mod calculator;
mod engine;
mod mahjongsoul;
mod replay;

const MSC_PORT: u32 = 52000;
const GUI_PORT: u32 = 52001;

pub use calculator::CalculatorApp;
pub use engine::EngineApp;
pub use mahjongsoul::MahjongsoulApp;
pub use replay::ReplayApp;
