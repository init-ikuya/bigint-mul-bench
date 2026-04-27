//! 多倍長整数を Vec<u32> のリトルエンディアンで表現する。
//! 各要素は 0..BASE の範囲。BASE は u32 に収まる範囲で設定。
//! 実用の BigInt ライブラリでは u64 の半分などを使うが、
//! ここではアルゴリズムの比較が目的なのでシンプルに。

pub const BASE: u64 = 1_000_000_000; // 10^9

/// 先頭のゼロを除去
pub fn trim(v: &mut Vec<u32>) {
    while v.len() > 1 && *v.last().unwrap() == 0 {
        v.pop();
    }
}

/// ゼロ埋めして長さを揃える
pub fn pad(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let n = a.len().max(b.len());
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.resize(n, 0);
    b.resize(n, 0);
    (a, b)
}

/// 多倍長の加算 (a + b)
pub fn add(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len().max(b.len());
    let mut result = Vec::with_capacity(n + 1);
    let mut carry: u64 = 0;
    for i in 0..n {
        let ai = if i < a.len() { a[i] as u64 } else { 0 };
        let bi = if i < b.len() { b[i] as u64 } else { 0 };
        let sum = ai + bi + carry;
        result.push((sum % BASE) as u32);
        carry = sum / BASE;
    }
    if carry > 0 {
        result.push(carry as u32);
    }
    result
}

/// 多倍長の減算 (a - b), a >= b を仮定
pub fn sub(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(a.len());
    let mut borrow: i64 = 0;
    for i in 0..a.len() {
        let ai = a[i] as i64;
        let bi = if i < b.len() { b[i] as i64 } else { 0 };
        let mut diff = ai - bi - borrow;
        if diff < 0 {
            diff += BASE as i64;
            borrow = 1;
        } else {
            borrow = 0;
        }
        result.push(diff as u32);
    }
    trim(&mut result);
    result
}

/// B^m 倍する（m 個のゼロを先頭に挿入）
pub fn shift(a: &[u32], m: usize) -> Vec<u32> {
    let mut result = vec![0u32; m];
    result.extend_from_slice(a);
    result
}

/// ランダムな n 桁（BASE進）の数を生成
/// seed を変えることで異なる値を生成できる（a と b で同一値にならないようにする）
pub fn random_digits(n: usize) -> Vec<u32> {
    random_digits_with_seed(n, 42)
}

pub fn random_digits_with_seed(n: usize, seed: u64) -> Vec<u32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut digits = Vec::with_capacity(n);
    for i in 0..n {
        let mut hasher = DefaultHasher::new();
        (i, n, seed).hash(&mut hasher);
        let h = hasher.finish();
        digits.push((h % BASE) as u32);
    }
    if let Some(last) = digits.last_mut()
        && *last == 0
    {
        *last = 1;
    }
    digits
}
