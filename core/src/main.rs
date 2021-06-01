mod app;
mod controller;
mod hand;
mod model;
mod operator;
mod util;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Mode is not specified");
        return;
    }

    let args2 = args[2..].to_vec();
    match args[1].as_str() {
        "J" => {
            // Jantama (雀魂botモード)
            app::mahjongsoul::App::new(args2).run();
        }
        "E" => {
            // Engine (bot対戦シミュレーションモード)
            app::engine::App::new(args2).run();
        }
        m => {
            println!("Unknown mode: {}", m)
        }
    }
}
