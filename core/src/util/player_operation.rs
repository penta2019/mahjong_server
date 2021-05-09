use std::fmt;

use crate::model::*;

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerOperation {
    Nop,                     // キャンセル (鳴き,ロンのスキップ)
    Discard(Vec<Tile>),      // 打牌 (配列はチー後に捨てることができない牌)
    Chii(Vec<(Tile, Tile)>), // チー (配列は鳴きが可能な組み合わせ 以下同様)
    Pon(Vec<(Tile, Tile)>),  // ポン
    Ankan(Vec<Tile>),        // 暗槓
    Minkan(Vec<Tile>),       // 明槓
    Kakan(Vec<Tile>),        // 加槓
    Riichi(Vec<Tile>),       // リーチ
    Tsumo,                   // ツモ
    Ron,                     // ロン
    Kyushukyuhai,            // 九種九牌
    Kita,                    // 北抜き
}

pub fn enc_discard(t: Tile, m: bool) -> usize {
    t.0 * 10 + t.1 + if m { 100 } else { 0 }
}

pub fn dec_discard(i: usize) -> (Tile, bool) {
    let m = i / 100 == 1;
    let ti = (i / 10) % 10;
    let ni = i % 10;
    (Tile(ti, ni), m)
}

pub trait Operator: OperatorClone + Send {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        operatons: &Vec<PlayerOperation>,
    ) -> (Index, Index);
    fn debug_string(&self) -> String;
}

impl fmt::Debug for dyn Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Algo{{{}}}", self.debug_string())
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

// NullOperator
#[derive(Clone)]
pub struct NullOperator {}

impl NullOperator {
    pub fn new() -> Self {
        NullOperator {}
    }
}

impl Operator for NullOperator {
    fn handle_operation(
        &mut self,
        _stage: &Stage,
        _seat: Seat,
        _operatons: &Vec<PlayerOperation>,
    ) -> (Index, Index) {
        panic!();
    }

    fn debug_string(&self) -> String {
        "NullOperator".to_string()
    }
}
