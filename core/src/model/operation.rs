use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PlayerOperationType {
    Nop, // Operator用 Turn: ツモ切り(主にリーチ中), Call: 鳴き,ロンのスキップ

    // Turn Operations
    Discard,      // 打牌 (ゲーム側から提供される配列は鳴き後に捨てられない配)
    Ankan,        // 暗槓
    Kakan,        // 加槓
    Riichi,       // リーチ
    Tsumo,        // ツモ
    Kyushukyuhai, // 九種九牌
    Kita,         // 北抜き

    // Call Operations
    Chii,   // チー (配列は鳴きが可能な組み合わせ 以下同様)
    Pon,    // ポン
    Minkan, // 明槓
    Ron,    // ロン
}

pub type OpType = PlayerOperationType;

// Vec<Tile>は操作により手牌からなくなる牌
// Chii, Ponなどの標的の牌はstage.last_tileを参照する
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerOperation(pub PlayerOperationType, pub Vec<Tile>);

impl PlayerOperation {
    #[inline]
    pub fn nop() -> Self {
        Self(OpType::Nop, vec![])
    }
    #[inline]
    pub fn discard(t: Tile) -> Self {
        Self(OpType::Discard, vec![t])
    }
    #[inline]
    pub fn ankan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 4);
        v.sort();
        Self(OpType::Ankan, v)
    }
    #[inline]
    pub fn kakan(t: Tile) -> Self {
        Self(OpType::Kakan, vec![t])
    }
    #[inline]
    pub fn riichi(t: Tile) -> Self {
        Self(OpType::Riichi, vec![t])
    }
    #[inline]
    pub fn tsumo() -> Self {
        Self(OpType::Tsumo, vec![])
    }
    #[inline]
    pub fn kyushukyuhai() -> Self {
        Self(OpType::Kyushukyuhai, vec![])
    }
    #[inline]
    pub fn kita() -> Self {
        Self(OpType::Kita, vec![Tile(TZ, WN)])
    }
    #[inline]
    pub fn chii(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(OpType::Chii, v)
    }
    #[inline]
    pub fn pon(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(OpType::Pon, v)
    }
    #[inline]
    pub fn minkan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 3);
        v.sort();
        Self(OpType::Minkan, v)
    }
    #[inline]
    pub fn ron() -> Self {
        Self(OpType::Ron, vec![])
    }
}

pub type Op = PlayerOperation;
