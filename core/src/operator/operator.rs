use std::fmt;

use crate::model::*;
use crate::util::stage_listener::StageListener;

use PlayerOperationType::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerOperationType {
    Nop, // Turn: ツモ切り(主にリーチ中), Call: 鳴き,ロンのスキップ
    // Turn Operations
    Discard,      // 打牌 (配列はチー後に捨てることができない牌)
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

// Vec<Tile>は操作により手牌からなくなる牌
// Chii, Ponなどの標的の牌はstage.last_tileを参照する
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerOperation(pub PlayerOperationType, pub Vec<Tile>);
impl PlayerOperation {
    #[inline]
    pub fn nop() -> Self {
        Self(Nop, vec![])
    }
    #[inline]
    pub fn discard(t: Tile) -> Self {
        Self(Discard, vec![t])
    }
    #[inline]
    pub fn ankan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 4);
        v.sort();
        Self(Ankan, v)
    }
    #[inline]
    pub fn kakan(t: Tile) -> Self {
        Self(Kakan, vec![t])
    }
    #[inline]
    pub fn riichi(t: Tile) -> Self {
        Self(Riichi, vec![t])
    }
    #[inline]
    pub fn tsumo() -> Self {
        Self(Tsumo, vec![])
    }
    #[inline]
    pub fn kyushukyuhai() -> Self {
        Self(Kyushukyuhai, vec![])
    }
    #[inline]
    pub fn kita() -> Self {
        Self(Kita, vec![Tile(TZ, WN)])
    }
    #[inline]
    pub fn chii(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(Chii, v)
    }
    #[inline]
    pub fn pon(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 2);
        v.sort();
        Self(Pon, v)
    }
    #[inline]
    pub fn minkan(mut v: Vec<Tile>) -> Self {
        assert!(v.len() == 3);
        v.sort();
        Self(Minkan, v)
    }
    #[inline]
    pub fn ron() -> Self {
        Self(Ron, vec![])
    }
}

pub type Op = PlayerOperation;

// Operator trait
pub trait Operator: StageListener + OperatorClone + Send {
    fn set_seat(&mut self, _: Seat) {}
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation;
    fn name(&self) -> String;
}

impl fmt::Debug for dyn Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait OperatorClone {
    fn clone_box(&self) -> Box<dyn Operator>;
}

impl<T> OperatorClone for T
where
    T: 'static + Operator + Clone,
{
    fn clone_box(&self) -> Box<dyn Operator> {
        Box::new(self.clone())
    }
}
