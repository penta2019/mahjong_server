use std::fmt;

use serde_json::Value;

use crate::error;

pub type Res<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub fn next_value<T>(it: &mut std::slice::Iter<'_, std::string::String>, opt: &str) -> T
where
    T: std::str::FromStr,
    T::Err: fmt::Display,
{
    let n = it
        .next()
        .unwrap_or_else(|| error_exit(format!("{}: value missing", opt)));
    n.parse()
        .unwrap_or_else(|e| error_exit(format!("{}: {} '{}'", opt, e, n)))
}

pub fn sleep(sec: f64) {
    std::thread::sleep(std::time::Duration::from_millis((sec * 1000.0) as u64));
}

pub fn unixtime_now() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
        / 1000.0
}

pub fn prompt() -> String {
    use std::io::{stdin, stdout, Write};
    print!("> ");
    stdout().flush().unwrap();
    let mut buf = String::new();
    stdin().read_line(&mut buf).ok();
    buf
}

pub fn flush() {
    use std::io::{stdout, Write};
    stdout().flush().unwrap();
}

pub fn error_exit<T: fmt::Display, U>(t: T) -> U {
    error!("{}", t);
    std::process::exit(1);
}

pub fn get_paths(dir: &std::path::Path) -> Res<Vec<std::path::PathBuf>> {
    let mut entries = std::fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

pub fn write_to_file(file_path: &str, data: &str) -> Res {
    use std::io::Write;
    let path = std::path::Path::new(file_path);
    let prefix = path.parent().ok_or("invalid path")?;
    std::fs::create_dir_all(prefix)?;
    let mut f = std::fs::File::create(path)?;
    write!(f, "{}", data)?;
    Ok(())
}

// pub fn vec_remove<T: PartialEq>(v: &mut Vec<T>, e: &T) {
//     v.remove(v.iter().position(|x| x == e).unwrap());
// }

pub fn vec_count<T: PartialEq>(v: &[T], e: &T) -> usize {
    v.iter().filter(|&n| n == e).count()
}

pub fn vec_to_string<T: fmt::Display>(v: &[T]) -> String {
    let vs: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
    "[".to_string() + &vs.join(", ") + "]"
}

pub fn cartesian_product<T>(vv: &[Vec<T>]) -> Vec<Vec<&T>> {
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
pub fn rank_by_index_vec<T: Ord + Clone>(v: &[T]) -> Vec<usize> {
    let mut i_n: Vec<(usize, &T)> = v.iter().enumerate().collect();
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
pub fn rank_by_rank_vec<T: Ord + Clone>(v: &[T]) -> Vec<usize> {
    let mut res = vec![0; v.len()];
    for (r, &i) in rank_by_index_vec(v).iter().enumerate() {
        res[i] = r;
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

pub fn as_array(v: &Value) -> &[Value] {
    v.as_array().unwrap()
}

pub fn as_enumerate(v: &Value) -> impl Iterator<Item = (usize, &Value)> {
    v.as_array().unwrap().iter().enumerate()
}

pub fn as_vec<T>(f: impl Fn(&Value) -> T, v: &Value) -> Vec<T> {
    as_array(v).iter().map(f).collect()
}
