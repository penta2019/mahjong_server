use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    // Controller側から提供されるDiscard(打牌)の配列は鳴き後に捨てられない牌の一覧
    // 打牌は空切り優先. 明示的にツモ切りを行いたい場合Nopを返す Action(Nop, [])
    // Controller側から提供されるRiichi(リーチ)の配列はリーチ宣言可能な牌の一覧
    // リーチは空切り優先. 明示的にツモ切りリーチを行いたい場合は空の配列を返す Action(Riichi, [])
    // DiscardとRiichi以外は提供されたActionの配列から厳密に同じものを返却
    Nop, // Turn: ツモ切り(主にリーチ中), Call: 鳴き,ロンのスキップ

    // Turn Actions
    Discard,      // 打牌
    Riichi,       // リーチ
    Ankan,        // 暗槓
    Kakan,        // 加槓
    Tsumo,        // ツモ
    Kyushukyuhai, // 九種九牌
    Nukidora,     // 北抜き

    // Call Actions (配列は鳴きにより手牌から消失する牌のリスト)
    Chi,    // チー
    Pon,    // ポン
    Minkan, // 明槓
    Ron,    // ロン
}

// Vec<Tile>は操作により手牌からなくなる牌
// Chi, Ponなどの標的の牌はstage.last_tileを参照する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub action_type: ActionType,
    pub tiles: Vec<Tile>,
}

impl Action {
    #[inline]
    pub fn new(action_type: ActionType, tiles: Vec<Tile>) -> Self {
        Self { action_type, tiles }
    }

    #[inline]
    pub fn nop() -> Self {
        Self::new(ActionType::Nop, vec![])
    }

    #[inline]
    pub fn discard(t: Tile) -> Self {
        Self::new(ActionType::Discard, vec![t])
    }

    #[inline]
    pub fn ankan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 4);
        v.sort();
        Self::new(ActionType::Ankan, v)
    }

    #[inline]
    pub fn kakan(t: Tile) -> Self {
        Self::new(ActionType::Kakan, vec![t])
    }

    #[inline]
    pub fn riichi(t: Tile) -> Self {
        Self::new(ActionType::Riichi, vec![t])
    }

    #[inline]
    pub fn riichi_drawn() -> Self {
        Self::new(ActionType::Riichi, vec![])
    }

    #[inline]
    pub fn tsumo() -> Self {
        Self::new(ActionType::Tsumo, vec![])
    }

    #[inline]
    pub fn kyushukyuhai() -> Self {
        Self::new(ActionType::Kyushukyuhai, vec![])
    }

    #[inline]
    pub fn nukidora() -> Self {
        Self::new(ActionType::Nukidora, vec![Tile(TZ, WN)])
    }

    #[inline]
    pub fn chi(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self::new(ActionType::Chi, v)
    }

    #[inline]
    pub fn pon(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self::new(ActionType::Pon, v)
    }

    #[inline]
    pub fn minkan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 3);
        v.sort();
        Self::new(ActionType::Minkan, v)
    }
    #[inline]
    pub fn ron() -> Self {
        Self::new(ActionType::Ron, vec![])
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.action_type, self.tiles)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinningTile {
    pub tile: Tile,     // 和了牌
    pub has_yaku: bool, // 出和了可能な役があるかどうか
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenpai {
    pub discard_tile: Tile,              // 聴牌になる打牌
    pub winning_tiles: Vec<WinningTile>, // 聴牌になる和了牌のリスト
    pub is_furiten: bool,                // フリテンの有無
}

// 用途不明
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "type")]
// pub struct PossibleActions {
//     pub actions: Vec<Action>,
//     pub tenpais: Vec<Tenpai>,
// }
