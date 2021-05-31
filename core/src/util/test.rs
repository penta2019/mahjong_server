#[allow(dead_code)]
fn silence_unused_warning() {
    let _ = crate::model::Tile::is_simple;
    let _ = crate::hand::win::is_normal_win;
    let _ = crate::hand::win::is_chiitoitsu_win;
}

#[test]
fn test_hand1() {
    use crate::hand::evaluate::evaluate_hand;
    use crate::hand::yaku::YakuFlags;
    use crate::model::*;

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
