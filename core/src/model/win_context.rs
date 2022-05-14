use super::*;

pub type Points = (Point, Point, Point); // (ロンの支払い, ツモ・子の支払い, ツモ・親の支払い)

#[derive(Debug, Deserialize, Serialize)]
pub struct WinContext {
    pub hand: Vec<Tile>,             // 和了手牌(鳴きは含まない)
    pub yakus: Vec<(String, usize)>, // 役一覧(ドラを含む), Vec<(name, fan)>
    pub fu: usize,                   // 符数
    pub fan: usize,                  // 飜数(ドラを含む), 役満倍率(is_yakuman=trueの時)
    pub yakuman_times: usize,        // 役満倍率 (0: 通常役, 1: 役満, 2: 二倍役満, ...)
    pub score_title: String,         // 倍満, 跳満, ...
    pub points: Points,              // 支払い得点
}
