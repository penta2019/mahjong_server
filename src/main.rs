#![warn(rust_2018_idioms)]
// 構造的な意味合いや一貫性を保つために以下の警告は無効化
#![allow(clippy::useless_format)]
#![allow(clippy::get_first)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::single_match)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::too_many_arguments)]

mod actor;
mod app;
mod convert;
mod hand;
mod listener;
mod model;
mod tool;
mod util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        error!("mode not specified");
        return;
    }

    let args2 = args[2..].to_vec();
    match args[1].as_str() {
        "C" => {
            // Calculator (役計算モード)
            app::CalculatorApp::new(args2).run();
        }
        "E" => {
            // Engine (bot対戦シミュレーションモード)
            app::EngineApp::new(args2).run();
        }
        "J" => {
            // Jantama (雀魂botモード)
            app::MahjongsoulApp::new(args2).run();
        }
        "R" => {
            // Replay (牌譜リプレイモード)
            app::ReplayApp::new(args2).run();
        }
        m => {
            error!("unknown mode: {}", m)
        }
    }
}
