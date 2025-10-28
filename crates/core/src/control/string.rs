use super::common::calc_seat_offset;
use crate::{
    model::*,
    util::misc::{Res, vec_count},
};

pub fn tile_type_from_char(ch: char) -> Res<Type> {
    match ch {
        'm' => Ok(TM),
        'p' => Ok(TP),
        's' => Ok(TS),
        'z' => Ok(TZ),
        _ => Err(format!("invalid tile type char: {ch}"))?,
    }
}

pub fn tile_type_to_char(ti: Type) -> char {
    match ti {
        TM => 'm',
        TP => 'p',
        TS => 's',
        TZ => 'z',
        _ => panic!("invalid tile type index: {ti}"),
    }
}

pub fn tile_number_from_char(ch: char) -> Res<Tnum> {
    if let Some(i) = ch.to_digit(10) {
        Ok(i as Tnum)
    } else {
        Err(format!("invalid tile number char: {ch}"))?
    }
}

pub fn tile_number_to_char(ni: Tnum) -> char {
    std::char::from_digit(ni as u32, 10)
        .unwrap_or_else(|| panic!("invalid tile number index: {}", ni))
}

pub fn wind_from_char(ch: char) -> Res<Tnum> {
    Ok(match ch {
        'E' => WE,
        'S' => WS,
        'W' => WW,
        'N' => WN,
        _ => Err(format!("invalid wind char: {}", ch))?,
    })
}

pub fn wind_to_char(ni: Tnum) -> char {
    match ni {
        WE => 'E',
        WS => 'S',
        WW => 'W',
        WN => 'N',
        _ => panic!("invalid wind index: {ni}"),
    }
}

pub fn tiles_from_string(exp: &str) -> Res<Vec<Tile>> {
    let mut tiles = vec![];
    let undef: usize = 255; // TODO: Opitonに置き換え
    let mut ti = undef;
    for ch in exp.chars() {
        match ch {
            'm' | 'p' | 's' | 'z' => ti = tile_type_from_char(ch).unwrap(),
            '0'..='9' => {
                if ti == undef {
                    Err("tile number befor tile type")?;
                }
                let ni = ch.to_digit(10).unwrap() as usize;
                tiles.push(Tile(ti, ni));
            }
            _ => {
                Err(format!("invalid char: '{}'", ch))?;
            }
        }
    }
    Ok(tiles)
}

pub fn tiles_to_string(tiles: &[Tile]) -> String {
    let mut res = String::new();
    let mut last_ti = 255;
    for t in tiles {
        if t.0 != last_ti {
            last_ti = t.0;
            res.push(tile_type_to_char(t.0));
        }
        res.push_str(&t.1.to_string());
    }
    res
}

pub fn meld_from_string(exp: &str) -> Res<Meld> {
    let undef: usize = 255;
    let seat = 0; // 点数計算する上で座席の番号は関係ないので0で固定
    let mut ti = undef;
    let mut nis = vec![];

    let mut from = 0;
    let mut tiles = vec![];
    let mut froms = vec![];
    for ch in exp.chars() {
        match ch {
            'm' | 'p' | 's' | 'z' => ti = tile_type_from_char(ch).unwrap(),
            '+' => {
                if froms.is_empty() {
                    Err("invalid '+' suffix")?;
                }
                let last = froms.len() - 1;
                froms[last] = from % SEAT;
            }
            '0'..='9' => {
                if ti == undef {
                    Err("tile number befor tile type")?;
                }

                from += 1;
                let ni = ch.to_digit(10).unwrap() as usize;
                nis.push(if ni == 0 { 5 } else { ni });
                tiles.push(Tile(ti, ni));
                froms.push(seat);
            }
            _ => {
                Err(format!("invalid char: '{}'", ch))?;
            }
        }
    }

    nis.sort();
    let mut diffs = vec![];
    let mut ni0 = nis[0];
    for ni in &nis[1..] {
        diffs.push(ni - ni0);
        ni0 = *ni;
    }

    let meld_type = if diffs.len() == 2 && vec_count(&diffs, &1) == 2 {
        MeldType::Chi
    } else if diffs.len() == 2 && vec_count(&diffs, &0) == 2 {
        MeldType::Pon
    } else if diffs.len() == 3 && vec_count(&diffs, &0) == 3 {
        if vec_count(&froms, &seat) == 4 {
            MeldType::Ankan
        } else {
            MeldType::Minkan
        }
    } else {
        Err(format!("invalid meld: '{}'", exp))?
    };

    Ok(Meld {
        step: 0,
        meld_type,
        tiles,
        froms,
    })
}

pub fn meld_to_string(m: &Meld, s: Seat) -> String {
    let mut tiles = Vec::new();
    let mut m_t = Z8;
    let mut m_f = NO_SEAT;
    for (&t, &f) in m.tiles.iter().zip(&m.froms) {
        if f == s {
            tiles.push((t, f));
        } else {
            m_t = t;
            m_f = f;
        }
    }
    if m_t != Z8 {
        let pos = 3 - calc_seat_offset(s, m_f);
        tiles.insert(pos, (m_t, m_f));
    }

    let mut res = String::new();
    res.push(tile_type_to_char(m.tiles[0].0));
    for (t, f) in tiles {
        res.push(tile_number_to_char(t.1));
        if f != s {
            res.push('+');
        }
    }
    res
}

#[test]
fn test_tiletable() {
    use super::common::{tiles_from_tile_table, tiles_to_tile_table};
    let hand_str = "p34777s1230567z66";
    let hand = tiles_from_string(&hand_str).unwrap();
    let tt = tiles_to_tile_table(&hand);
    let hand2 = tiles_from_tile_table(&tt);
    assert_eq!(hand, hand2);
}

#[test]
fn test_tiles_to_string() {
    let hand_str = "p34777s1230567z66";
    let hand = tiles_from_string(&hand_str).unwrap();
    let hand_str2 = tiles_to_string(&hand);
    assert_eq!(hand_str, hand_str2);
}

#[test]
fn test_meld_to_string() {
    let s = 3;
    let m = Meld {
        step: 0,
        meld_type: MeldType::Kakan,
        tiles: tiles_from_string("m0m5m5m5").unwrap(),
        froms: vec![0, 3, 3, 3],
    };
    println!("{}", meld_to_string(&m, s));
}
