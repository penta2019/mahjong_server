use super::*;

pub type Points = (Point, Point, Point); // (ロンの支払い, ツモ・子の支払い, ツモ・親の支払い)

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Yaku {
    pub name: String,
    pub fan: usize,
}

// 手役評価関数 hand::evaluate::hand_evaluateの返り値
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreContext {
    pub yakus: Vec<Yaku>, // 役一覧(ドラを含む), Vec<(name, fan)>
    pub fu: usize,        // 符数
    pub fan: usize,       // 飜数(ドラを含む), 役満の場合は0
    pub yakuman: usize,   // 役満倍率 (0: 通常役, 1: 役満, 2: 二倍役満, ...)
    pub score: Score,     // 和了得点
    pub points: Points,   // 支払い得点の内訳
    pub title: String,    // 倍満, 跳満, ...
}

// 和了情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinContext {
    pub seat: usize,                 // 和了者
    pub hand: Vec<Tile>,             // 手牌 (和了牌は含まない = 3*n+2-1枚)
    pub winning_tile: Tile,          // 和了牌
    pub melds: Vec<Meld>,            // 副露
    pub is_dealer: bool,             // 親番フラグ
    pub is_drawn: bool,              // ツモフラグ
    pub is_riichi: bool,             // 立直フラグ
    pub pao: Option<Seat>,           // 責任払い
    pub delta_scores: [Point; SEAT], // この和了による点数変動 (ダブロン時の他の和了者の点数は含まない)
    pub score_context: ScoreContext, // スコア計算に関する情報
}
