use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    Nop, // Actor用 Turn: ツモ切り(主にリーチ中), Call: 鳴き,ロンのスキップ

    // Turn Actions
    Discard,      // 打牌 (ゲーム側から提供される配列は鳴き後に捨てられない配)
    Ankan,        // 暗槓
    Kakan,        // 加槓
    Riichi,       // リーチ
    Tsumo,        // ツモ
    Kyushukyuhai, // 九種九牌
    Kita,         // 北抜き

    // Call Actions
    Chi,    // チー (配列は鳴きが可能な組み合わせ 以下同様)
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
