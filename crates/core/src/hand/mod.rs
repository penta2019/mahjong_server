// 手牌の役や点数計算を行うモジュール
mod evaluate;
mod parse;
mod point;
mod win;
mod yaku;

pub use self::{
    evaluate::{evaluate_hand, evaluate_hand_ron, evaluate_hand_tsumo},
    parse::SetPairType,
    win::{calc_discards_to_win, calc_tiles_to_normal_win, calc_tiles_to_win, is_normal_win},
    yaku::{YakuDefine, YakuFlags},
};
