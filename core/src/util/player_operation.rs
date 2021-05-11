use std::fmt;

use crate::model::*;

use PlayerOperation::*;
use TileStateType::*;

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerOperation {
    Nop, // Turn: ツモ切り(主にリーチ中), Call: 鳴き,ロンのスキップ
    // Turn Operations
    Discard(Vec<Tile>), // 打牌 (配列はチー後に捨てることができない牌)
    Ankan(Vec<Tile>),   // 暗槓
    Kakan(Vec<Tile>),   // 加槓
    Riichi(Vec<Tile>),  // リーチ
    Tsumo,              // ツモ
    Kyushukyuhai,       // 九種九牌
    Kita,               // 北抜き
    // Call Operations
    Chii(Vec<(Tile, Tile)>), // チー (配列は鳴きが可能な組み合わせ 以下同様)
    Pon(Vec<(Tile, Tile)>),  // ポン
    Minkan(Vec<Tile>),       // 明槓
    Ron,                     // ロン
}

// Operator trait
pub trait Operator: OperatorClone + Send {
    fn handle_operation(
        &mut self,
        stage: &Stage,
        seat: Seat,
        operatons: &Vec<PlayerOperation>,
    ) -> PlayerOperation;
    fn debug_string(&self) -> String {
        "Operator".to_string()
    }
}

impl fmt::Debug for dyn Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.debug_string())
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
    ) -> PlayerOperation {
        panic!();
    }

    fn debug_string(&self) -> String {
        "NullOperator".to_string()
    }
}

// util
pub fn count_left_tile(stage: &Stage, seat: Seat, tile: Tile) -> usize {
    let mut n = 0;
    for &st in &stage.tile_states[tile.0][tile.1] {
        match st {
            U => {
                n += 1;
            }
            H(s) => {
                if s != seat {
                    n += 1;
                }
            }
            _ => {}
        }
    }
    n
}

macro_rules! op_without_arg {
    ($x:pat, $ops:expr) => {
        for (op_idx, op2) in $ops.iter().enumerate() {
            if let $x = op2 {
                return (op_idx, 0);
            }
        }
    };
}

macro_rules! op_with_arg {
    ($x:ident, $ops:expr, $v:expr) => {{
        for (op_idx, op2) in $ops.iter().enumerate() {
            if let $x(v2) = op2 {
                for (arg_idx, &e) in v2.iter().enumerate() {
                    if e == $v[0] {
                        return (op_idx, arg_idx);
                    }
                }
            }
        }
    }};
}

pub fn get_op_idx(ops: &Vec<PlayerOperation>, op: &PlayerOperation) -> (usize, usize) {
    match op {
        Nop => op_without_arg!(Nop, ops),
        Discard(_) => return (0, 0),
        Ankan(v) => op_with_arg!(Ankan, ops, v),
        Kakan(v) => op_with_arg!(Kakan, ops, v),
        Riichi(v) => op_with_arg!(Riichi, ops, v),
        Tsumo => op_without_arg!(Tsumo, ops),
        Kyushukyuhai => op_without_arg!(Kyushukyuhai, ops),
        Kita => op_without_arg!(Kita, ops),
        Chii(v) => op_with_arg!(Chii, ops, v),
        Pon(v) => op_with_arg!(Pon, ops, v),
        Minkan(v) => op_with_arg!(Minkan, ops, v),
        Ron => op_without_arg!(Ron, ops),
    }
    panic!("Operation '{:?}' not found id '{:?}'", op, ops);
}
