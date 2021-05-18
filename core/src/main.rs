mod app;
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
        "H" => {
            // Hand Yaku (手役評価モード)
            app::hand_yaku::App::new(args2).run()
        }
        m => {
            println!("Unknown mode: {}", m)
        }
    }
}

#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = hand::win::is_normal_win;
    let _ = hand::win::is_chiitoitsu_win;
    let _ = model::Tile::is_simple;
    let _ = hand::win::calc_tiles_to_chiitoitsu_win;
}

#[test]
fn test_hand1() {
    use hand::evaluate::evaluate_hand;
    use hand::yaku::YakuFlags;
    use model::*;

    let hand = [
        [0, 0, 0, 0, 0, 0, 0, 1, 1, 1],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 2, 2, 2, 2],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let melds = vec![Meld {
        step: 0,
        seat: 0,
        type_: MeldType::Chii,
        tiles: vec![Tile(TP, 7); 4],
        froms: vec![0; 4],
    }];
    let doras = vec![Tile(TP, 9), Tile(TP, 1), Tile(TP, 2)];
    let winnig_tile = Tile(TM, 1);
    let is_self_drawn = false;
    let is_leader = true;
    let yaku = YakuFlags::default();
    // yaku.menzentsumo = true;
    // yaku.riichi = true;
    // yaku.ippatsu = true;

    let start = std::time::Instant::now();

    // for _ in 0..1000 {
    //     evaluate_hand(
    //         &hand,
    //         &melds,
    //         &doras,
    //         winnig_tile,
    //         is_self_drawn,
    //         is_leader,
    //         WW,
    //         WE,
    //         yaku,
    //     );
    // }

    let res = evaluate_hand(
        &hand,
        &melds,
        &doras,
        &vec![],
        winnig_tile,
        is_self_drawn,
        is_leader,
        WW,
        WE,
        yaku,
    );

    let elapsed = start.elapsed();

    println!("{:?}", res);
    println!("{}ms", elapsed.as_nanos() as f32 / 1000000.0);
}
