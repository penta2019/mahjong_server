use crate::hand::parse::*;
use crate::model::*;

use SetPairType::*;

// Block Info
#[derive(Debug)]
pub struct BlockInfo {
    pub tile: Tile, // ブロックのスタート位置
    pub len: usize, // ブロックの長さ
    pub num: usize, // ブロック内の牌の数
}

impl BlockInfo {
    fn new() -> Self {
        Self {
            tile: Tile(TZ, UK),
            len: 0,
            num: 0,
        }
    }
}

pub fn calc_block_info(hand: &TileTable) -> Vec<BlockInfo> {
    let mut vbi = vec![];
    let mut bi = BlockInfo::new();

    // 数牌
    for ti in 0..TZ {
        for ni in 1..TNUM {
            let c = hand[ti][ni];
            if bi.len == 0 {
                if c != 0 {
                    // ブロック始端
                    bi.tile = Tile(ti, ni);
                    bi.len = 1;
                    bi.num = c;
                }
            } else {
                if c == 0 {
                    if bi.tile.1 + bi.len < ni {
                        // ブロック終端
                        vbi.push(bi);
                        bi = BlockInfo::new();
                    }
                } else {
                    // ブロック延長
                    bi.len = ni - bi.tile.1 + 1;
                    bi.num += c;
                }
            }
        }
        if bi.len != 0 {
            vbi.push(bi);
            bi = BlockInfo::new();
        }
    }

    // 字牌
    for ni in 1..=DR {
        let c = hand[TZ][ni];
        if c != 0 {
            vbi.push(BlockInfo {
                tile: Tile(TZ, ni),
                len: 1,
                num: c,
            });
        }
    }

    vbi
}

fn parse_block_into_shuntsu(row: &TileRow, block: &BlockInfo) -> (Vec<SetPair>, TileRow) {
    let mut row = row.clone();
    let mut i = block.tile.1;
    let mut res = vec![];
    loop {
        let (ni0, ni1, ni2) = (i, i + 1, i + 2);
        if ni2 == block.tile.1 + block.len {
            break;
        }
        if row[ni0] > 0 && row[ni1] > 0 && row[ni2] > 0 {
            res.push(SetPair(Shuntsu, Tile(block.tile.0, ni0)));
            row[ni0] -= 1;
            row[ni1] -= 1;
            row[ni2] -= 1;
        } else {
            i += 1;
        }
    }
    (res, row)
}

fn parse_block_into_sets(row: &TileRow, block: &BlockInfo) -> Vec<(Vec<SetPair>, TileRow)> {
    let ni = block.tile.1;
    let mut koutsu_nis = vec![];
    let mut flag_end = 1;
    for ni2 in ni..ni + block.len {
        if row[ni2] >= 3 {
            koutsu_nis.push(ni2);
            flag_end *= 2;
        }
    }

    let mut res = vec![];
    let mut flags: usize = 0;
    loop {
        let mut row = row.clone();
        let mut sp = vec![];
        for (i, &ni) in koutsu_nis.iter().enumerate() {
            if (flags >> i) & 1 == 1 {
                row[ni] -= 3;
                sp.push(SetPair(Koutsu, Tile(block.tile.0, ni)));
            }
        }

        let (mut sp2, tr) = parse_block_into_shuntsu(&row, &block);
        sp.append(&mut sp2);
        res.push((sp, tr));

        flags += 1;
        if flags == flag_end {
            break;
        }
    }

    res
}

#[test]
fn test_block() {
    let row = [0, 3, 1, 1, 4, 0, 0, 0, 0, 0];
    let block = BlockInfo {
        tile: Tile(0, 1),
        len: 4,
        num: 8,
    };
    println!("{:?}", parse_block_into_sets(&row, &block));
}
