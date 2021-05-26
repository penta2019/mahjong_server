use std::fmt;

use serde_json::Value;

pub fn as_usize(v: &Value) -> usize {
    v.as_i64().unwrap() as usize
}

pub fn as_str(v: &Value) -> &str {
    v.as_str().unwrap()
}

pub fn vec_to_string<T: fmt::Display>(v: &Vec<T>) -> String {
    let vs: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
    vs.join(" ")
}

pub fn unixtime_now() -> f32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as f32
        / 1000.0
}

pub fn cartesian_product<'a, T>(vv: &'a Vec<Vec<T>>) -> Vec<Vec<&'a T>> {
    let lens: Vec<usize> = vv.iter().map(|l| l.len()).collect();
    let mut idxs = vec![0; vv.len()];
    let mut i = idxs.len() - 1;
    let mut res = vec![];
    loop {
        let mut v = vec![];
        for (i1, &i2) in idxs.iter().enumerate() {
            v.push(&vv[i1][i2]);
        }
        res.push(v);

        // increment idxs
        loop {
            if idxs[i] < lens[i] - 1 {
                idxs[i] += 1;
                i = idxs.len() - 1;
                break;
            } else {
                idxs[i] = 0;
                if i == 0 {
                    return res;
                }
            }
            i -= 1;
        }
    }
}

// 最も数字の大きい値のindexから順に格納した配列を返却
// 同じ値が複数ある場合, 先に入っていた要素のindexが先になる
pub fn rank_by_index_vec<T: Ord + Clone>(v: &Vec<T>) -> Vec<usize> {
    let mut i_n = vec![];
    for e in v.iter().enumerate() {
        i_n.push(e);
    }
    i_n.sort_by(|a, b| {
        if a.1 != b.1 {
            b.1.cmp(a.1)
        } else {
            a.0.cmp(&b.0)
        }
    });

    i_n.iter().map(|e| e.0).collect()
}

// 値が大きい順に並べた時に各要素が何番目であるかを示す配列を返却
pub fn rank_by_rank_vec<T: Ord + Clone>(v: &Vec<T>) -> Vec<usize> {
    let mut res = vec![0; v.len()];
    for e in rank_by_index_vec(v).iter().enumerate() {
        res[*e.1] = e.0;
    }
    res
}
