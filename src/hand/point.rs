use crate::model::{Point, Points};

// 親が他家を直撃した場合の点数表 (役満未満)
const POINT_DEALER: [[Point; 11]; 13] = [
    [0; 11],                                                      // 0飜
    [0, 0, 1500, 2000, 2400, 2900, 3400, 3900, 4400, 4800, 5300], // 1飜
    [
        2000, 2400, 2900, 3900, 4800, 5800, 6800, 7700, 8700, 9600, 10600,
    ], // 2飜
    [
        3900, 4800, 5800, 7700, 9600, 11600, 12000, 12000, 12000, 12000, 12000,
    ], // 3飜
    [
        7700, 9600, 11600, 12000, 12000, 12000, 12000, 12000, 12000, 12000, 12000,
    ], // 4飜
    [12000; 11],                                                  // 5飜
    [18000; 11],                                                  // 6飜
    [18000; 11],                                                  // 7飜
    [24000; 11],                                                  // 8飜
    [24000; 11],                                                  // 9飜
    [24000; 11],                                                  // 10飜
    [36000; 11],                                                  // 11飜
    [36000; 11],                                                  // 12飜
];

// 子が他家を直撃した場合の点数表 (役満未満)
const POINT_NON_DEALER: [[Point; 11]; 13] = [
    [0; 11],                                                      // 0飜
    [0, 0, 1000, 1300, 1600, 2000, 2300, 2600, 2900, 3200, 3600], // 1飜
    [
        1300, 1600, 2000, 2600, 3200, 3900, 4500, 5200, 5800, 6400, 7100,
    ], // 2飜
    [
        2600, 3200, 3900, 5200, 6400, 7700, 8000, 8000, 8000, 8000, 8000,
    ], // 3飜
    [
        5200, 6400, 7700, 8000, 8000, 8000, 8000, 8000, 8000, 8000, 8000,
    ], // 4飜
    [8000; 11],                                                   // 5飜
    [12000; 11],                                                  // 6飜
    [12000; 11],                                                  // 7飜
    [16000; 11],                                                  // 8飜
    [16000; 11],                                                  // 9飜
    [16000; 11],                                                  // 10飜
    [24000; 11],                                                  // 11飜
    [24000; 11],                                                  // 12飜
];

const POINT_YAKUMAN_DEALER: Point = 48000;
const POINT_YAKUMAN_NON_DEALER: Point = 32000;

fn calc_fu_index(fu: usize) -> usize {
    match fu {
        20 => 0,
        25 => 1,
        30 => 2,
        40 => 3,
        50 => 4,
        60 => 5,
        70 => 6,
        80 => 7,
        90 => 8,
        100 => 9,
        110 => 10,
        _ => panic!("invalid fu number"),
    }
}

fn ceil100(n: Point) -> Point {
    (n + 99) / 100 * 100
}

// 親の和了 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
fn get_points_dealer(fu: usize, fan: usize) -> Points {
    let fu_index = calc_fu_index(fu);
    let point = if fan < 13 {
        POINT_DEALER[fan][fu_index]
    } else {
        POINT_YAKUMAN_DEALER
    };
    (point, ceil100(point / 3), 0)
}

// 子の和了 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
fn get_points_non_dealer(fu: usize, fan: usize) -> Points {
    let fu_index = calc_fu_index(fu);
    let point = if fan < 13 {
        POINT_NON_DEALER[fan][fu_index]
    } else {
        POINT_YAKUMAN_NON_DEALER
    };
    (point, ceil100(point / 4), ceil100(point / 2))
}

// 親の役満 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
fn get_points_dealer_yakuman(mag: usize) -> Points {
    let s = POINT_YAKUMAN_DEALER * mag as i32;
    (s, s / 3, 0)
}

// 子の役満 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
fn get_points_non_dealer_yakuman(mag: usize) -> Points {
    let s = POINT_YAKUMAN_NON_DEALER * mag as i32;
    (s, s / 4, s / 2)
}

pub fn get_points(is_dealer: bool, fu: usize, fan: usize, yakuman_count: usize) -> Points {
    if is_dealer {
        if yakuman_count > 0 {
            get_points_dealer_yakuman(yakuman_count)
        } else {
            get_points_dealer(fu, fan)
        }
    } else {
        if yakuman_count > 0 {
            get_points_non_dealer_yakuman(yakuman_count)
        } else {
            get_points_non_dealer(fu, fan)
        }
    }
}

pub fn get_score_title(fu: usize, fan: usize, yakuman_count: usize) -> String {
    let fu_index = calc_fu_index(fu);
    match yakuman_count {
        0 => {
            if fan >= 13 {
                "数え役満"
            } else {
                match POINT_NON_DEALER[fan][fu_index] {
                    8000 => "満貫",
                    12000 => "跳満",
                    16000 => "倍満",
                    24000 => "三倍満",
                    _ => "",
                }
            }
        }
        1 => "役満",
        2 => "二倍役満",
        3 => "三倍役満",
        4 => "四倍役満",
        5 => "五倍役満",
        6 => "六倍役満",
        7 => "七倍役満",
        _ => panic!(),
    }
    .to_string()
}
