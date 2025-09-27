// 手牌の役や点数計算を行うモジュール
mod evaluate;
mod parse;
mod point;
mod win;
mod yaku;

pub use self::{
    evaluate::{evaluate_hand, evaluate_hand_ron, evaluate_hand_tsumo},
    point::get_score_title,
    win::{
        calc_discards_to_chiitoitsu_tenpai, calc_discards_to_kokushimusou_tenpai,
        calc_discards_to_normal_tenpai, calc_tiles_to_chiitoitsu_win,
        calc_tiles_to_kokushimusou_win, calc_tiles_to_normal_win, is_chiitoitsu_win,
        is_kokushimusou_win, is_normal_win,
    },
    yaku::{YakuDefine, YakuFlags},
};
