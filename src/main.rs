// 構造的な意味合いや一貫性を保つために以下のclippy警告は無効化
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
// guiを無効化してビルド(--no-default-features)した際のunuse警告を無効化
#![cfg_attr(not(feature = "gui"), allow(unused))]

// mod actor;
// mod app;
// mod control;
// mod convert;
// mod hand;
// mod listener;
// mod model;
// mod util;

// #[cfg(feature = "gui")]
// #[allow(clippy::type_complexity)]
// mod gui;

use mahjong_core::app;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 引数がない場合,バージョン情報を表示して終了
    if args.len() < 2 {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
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
        "R" => {
            // Replay (牌譜リプレイモード)
            app::ReplayApp::new(args2).run();
        }
        "G" => {
            // Gui (クライアントモード)
            app::ClientApp::new(args2).run();
        }
        m => {
            eprintln!("unknown mode: {}", m);
        }
    }
}
