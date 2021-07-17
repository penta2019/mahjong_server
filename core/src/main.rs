mod actor;
mod app;
mod controller;
mod convert;
mod hand;
mod model;
mod util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("[Error] Mode is not specified");
        return;
    }

    let args2 = args[2..].to_vec();
    match args[1].as_str() {
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
            println!("[Error] Unknown mode: {}", m)
        }
    }
}
