pub type PayScores = (i32, i32, i32); // (トータルの点数・ロン, ツモ・子の支払い, ツモ・親の支払い)

// 親が他家を直撃した場合の点数表 (役満未満)
const SCORE_LEADER: [[i32; 11]; 13] = [
    [0; 11],                                                      // 0翻
    [0, 0, 1500, 2000, 2400, 2900, 3400, 3900, 4400, 4800, 5300], // 1翻
    [
        2000, 2400, 2900, 3900, 4800, 5800, 6800, 7700, 8700, 9600, 10600,
    ], // 2翻
    [
        3900, 4800, 5800, 7700, 9600, 11600, 12000, 12000, 12000, 12000, 12000,
    ], // 3翻
    [
        7700, 9600, 11600, 12000, 12000, 12000, 12000, 12000, 12000, 12000, 12000,
    ], // 4翻
    [12000; 11],                                                  // 5翻
    [18000; 11],                                                  // 6翻
    [18000; 11],                                                  // 7翻
    [24000; 11],                                                  // 8翻
    [24000; 11],                                                  // 9翻
    [24000; 11],                                                  // 10翻
    [36000; 11],                                                  // 11翻
    [36000; 11],                                                  // 12翻
];

// 子が他家を直撃した場合の点数表 (役満未満)
const SCORE_NON_LEADER: [[i32; 11]; 13] = [
    [0; 11],                                                      // 0翻
    [0, 0, 1000, 1300, 1600, 2000, 2300, 2600, 2900, 3200, 3600], // 1翻
    [
        1300, 1600, 2000, 2600, 3200, 3900, 4500, 5200, 5800, 6400, 7100,
    ], // 2翻
    [
        2600, 3200, 3900, 5200, 6400, 7700, 8000, 8000, 8000, 8000, 8000,
    ], // 3翻
    [
        5200, 6400, 7700, 8000, 8000, 8000, 8000, 8000, 8000, 8000, 8000,
    ], // 4翻
    [8000; 11],                                                   // 5翻
    [12000; 11],                                                  // 6翻
    [12000; 11],                                                  // 7翻
    [16000; 11],                                                  // 8翻
    [16000; 11],                                                  // 9翻
    [16000; 11],                                                  // 10翻
    [24000; 11],                                                  // 11翻
    [24000; 11],                                                  // 12翻
];

const SCORE_YAKUMAN_LEADER: i32 = 48000;
const SCORE_YAKUMAN_NON_LEADER: i32 = 32000;

fn calc_fu_index(fu: i32) -> usize {
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

fn ceil100(n: i32) -> i32 {
    (n + 99) / 100 * 100
}

// 親の和了 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
pub fn get_pay_scores_leader(fu: i32, fan: usize) -> PayScores {
    let fu_index = calc_fu_index(fu);
    let score = if fan < 13 {
        SCORE_LEADER[fan][fu_index]
    } else {
        SCORE_YAKUMAN_LEADER
    };
    (score, ceil100(score / 3), 0)
}

// 子の和了 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
pub fn get_pay_scores_non_leader(fu: i32, fan: usize) -> PayScores {
    let fu_index = calc_fu_index(fu);
    let score = if fan < 13 {
        SCORE_NON_LEADER[fan][fu_index]
    } else {
        SCORE_YAKUMAN_NON_LEADER
    };
    (score, ceil100(score / 4), ceil100(score / 2))
}

// 親の役満 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
pub fn get_pay_scores_leader_yakuman(mag: usize) -> PayScores {
    let s = SCORE_YAKUMAN_LEADER * mag as i32;
    (s, s / 3, 0)
}

// 子の役満 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
pub fn get_pay_scores_non_leader_yakuman(mag: usize) -> PayScores {
    let s = SCORE_YAKUMAN_NON_LEADER * mag as i32;
    (s, s / 4, s / 2)
}

pub fn get_pay_scores(is_leader: bool, is_yakuman: bool, fu: i32, fan_mag: usize) -> PayScores {
    if is_leader {
        if is_yakuman {
            get_pay_scores_leader_yakuman(fan_mag)
        } else {
            get_pay_scores_leader(fu, fan_mag)
        }
    } else {
        if is_yakuman {
            get_pay_scores_non_leader_yakuman(fan_mag)
        } else {
            get_pay_scores_non_leader(fu, fan_mag)
        }
    }
}
