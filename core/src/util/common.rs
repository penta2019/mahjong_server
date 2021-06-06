use std::fmt;
use std::str::FromStr;

use serde_json::Value;

pub fn next_value<T: FromStr>(it: &mut std::slice::Iter<std::string::String>, emsg: &str) -> T
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    use std::process::exit;
    if let Some(n) = it.next() {
        return n.parse().unwrap();
    } else {
        println!("{}", emsg);
        exit(0);
    }
}

pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

pub fn unixtime_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn vec_remove<T: PartialEq>(v: &mut Vec<T>, e: &T) {
    v.remove(v.iter().position(|x| x == e).unwrap());
}

pub fn vec_to_string<T: fmt::Display>(v: &Vec<T>) -> String {
    let vs: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
    vs.join(" ")
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

pub fn as_usize(v: &Value) -> usize {
    v.as_i64().unwrap() as usize
}

pub fn as_i32(v: &Value) -> i32 {
    v.as_i64().unwrap() as i32
}

pub fn as_str(v: &Value) -> &str {
    v.as_str().unwrap()
}

pub fn as_bool(v: &Value) -> bool {
    v.as_bool().unwrap()
}

pub fn as_array(v: &Value) -> &Vec<serde_json::Value> {
    v.as_array().unwrap()
}

pub fn as_enumerate(v: &Value) -> std::iter::Enumerate<std::slice::Iter<'_, serde_json::Value>> {
    v.as_array().unwrap().iter().enumerate()
}
