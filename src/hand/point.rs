use crate::model::{Point, Points};

fn ceil(n: Point) -> Point {
    (n + 99) / 100 * 100
}

fn calc_base_point(fu: usize, fan: usize, yakuman: usize) -> Point {
    (if yakuman == 0 {
        let base = fu * 2_usize.pow(fan as u32 + 2);
        if base >= 2000 {
            match fan {
                ..6 => 2000,    // 満貫
                6..8 => 3000,   // 跳満
                8..11 => 4000,  // 倍満
                11..13 => 6000, // 三倍満
                13.. => 8000,   // 四倍満 (数え役満)
            }
        } else {
            base
        }
    } else {
        8000 * yakuman
    }) as i32
}

fn get_score_title(base_point: Point, yakuman: usize) -> String {
    match yakuman {
        0 => match base_point {
            2000 => "満貫",
            3000 => "跳満",
            4000 => "倍満",
            6000 => "三倍満",
            8000 => "数え役満",
            _ => "",
        },
        1 => "役満",
        2 => "二倍役満",
        3 => "三倍役満",
        4 => "四倍役満",
        5 => "五倍役満",
        6 => "六倍役満",
        7 => "七倍役満",
        _ => "N倍役満",
    }
    .to_string()
}

// 親の和了 (直撃, ツモ和了の子, ツモ和了の親)の支払いを返却
pub fn calc_points(is_dealer: bool, fu: usize, fan: usize, yakuman: usize) -> (Points, String) {
    let base = calc_base_point(fu, fan, yakuman);
    let title = get_score_title(base, yakuman);
    if is_dealer {
        ((ceil(base * 6), ceil(base * 2), 0), title)
    } else {
        ((ceil(base * 4), ceil(base), ceil(base * 2)), title)
    }
}

// cargo test --release print_points_table -- --nocapture
#[test]
fn print_points_table() {
    let fu_list = [20, 25, 30, 40, 50, 60, 70, 80, 90, 100, 110];

    println!("点数計算表 (子) ============================================");
    for fu in fu_list {
        print!("[{fu:3}符] ");
        for fan in 1..=4 {
            let (scores, _) = calc_points(false, fu, fan, 0);
            print!("{fan}飜:{:4}({:4}/{:4}) ", scores.0, scores.1, scores.2)
        }
        println!();
    }
    for fan in 5..=13 {
        let (scores, title) = calc_points(false, 20, fan, 0);
        println!(
            "{fan:2}飜:{:5}({:4}/{:5}) {title}",
            scores.0, scores.1, scores.2
        );
    }
    println!();

    println!("点数計算表 (親) ============================================");
    for fu in fu_list {
        print!("[{fu:3}符] ");
        for fan in 1..=4 {
            let (scores, _) = calc_points(true, fu, fan, 0);
            print!("{fan}飜:{:5}({:4}) ", scores.0, scores.1)
        }
        println!();
    }
    for fan in 5..=13 {
        let (scores, title) = calc_points(true, 20, fan, 0);
        println!("{fan:2}飜:{:5}({:5}) {title}", scores.0, scores.1);
    }
    println!();
}
