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
    Kita,         // 北抜き

    // Call Actions (配列は鳴きにより手牌から消失する牌のリスト)
    Chi,    // チー
    Pon,    // ポン
    Minkan, // 明槓
    Ron,    // ロン
}

// Vec<Tile>は操作により手牌からなくなる牌
// Chi, Ponなどの標的の牌はstage.last_tileを参照する
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action(pub ActionType, pub Vec<Tile>);

impl Action {
    #[inline]
    pub fn nop() -> Self {
        Self(ActionType::Nop, vec![])
    }

    #[inline]
    pub fn discard(t: Tile) -> Self {
        Self(ActionType::Discard, vec![t])
    }

    #[inline]
    pub fn ankan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 4);
        v.sort();
        Self(ActionType::Ankan, v)
    }

    #[inline]
    pub fn kakan(t: Tile) -> Self {
        Self(ActionType::Kakan, vec![t])
    }

    #[inline]
    pub fn riichi(t: Tile) -> Self {
        Self(ActionType::Riichi, vec![t])
    }

    #[inline]
    pub fn riichi_drawn() -> Self {
        Self(ActionType::Riichi, vec![])
    }

    #[inline]
    pub fn tsumo() -> Self {
        Self(ActionType::Tsumo, vec![])
    }

    #[inline]
    pub fn kyushukyuhai() -> Self {
        Self(ActionType::Kyushukyuhai, vec![])
    }

    #[inline]
    pub fn kita() -> Self {
        Self(ActionType::Kita, vec![Tile(TZ, WN)])
    }

    #[inline]
    pub fn chi(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(ActionType::Chi, v)
    }

    #[inline]
    pub fn pon(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(ActionType::Pon, v)
    }

    #[inline]
    pub fn minkan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 3);
        v.sort();
        Self(ActionType::Minkan, v)
    }
    #[inline]
    pub fn ron() -> Self {
        Self(ActionType::Ron, vec![])
    }
}
