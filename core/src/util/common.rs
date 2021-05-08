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

// pub fn vec_remove<T: PartialEq>(v: &mut Vec<T>, e: &T) {
//     v.remove(v.iter().position(|x| x == e).unwrap());
// }

pub fn unixtime_now() -> u64 {
    use std::time::*;
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
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
