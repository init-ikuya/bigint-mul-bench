/// 多倍長整数を Vec<u32> のリトルエンディアンで表現する。
/// 各要素は 0..BASE の範囲。BASE は u32 に収まる範囲で設定。
/// 実用の BigInt ライブラリでは u64 の半分などを使うが、
/// ここではアルゴリズムの比較が目的なのでシンプルに。

pub const BASE: u64 = 1_000_000_000; // 10^9

/// 先頭のゼロを除去
fn trim(v: &mut Vec<u32>) {
    while v.len() > 1 && *v.last().unwrap() == 0 {
        v.pop();
    }
}

/// ゼロ埋めして長さを揃える
fn pad(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let n = a.len().max(b.len());
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.resize(n, 0);
    b.resize(n, 0);
    (a, b)
}

// =========================================================
// 1. 筆算法 (Schoolbook) — O(n²)
// =========================================================

pub fn schoolbook(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len();
    let m = b.len();
    let mut result = vec![0u64; n + m];

    for i in 0..n {
        let mut carry: u64 = 0;
        for j in 0..m {
            let prod = a[i] as u64 * b[j] as u64 + result[i + j] + carry;
            result[i + j] = prod % BASE;
            carry = prod / BASE;
        }
        result[i + m] += carry;
    }

    let mut result: Vec<u32> = result.iter().map(|&x| x as u32).collect();
    trim(&mut result);
    result
}

// =========================================================
// 2. からつば法 (Karatsuba) — O(n^1.585)
// =========================================================

/// 多倍長の加算 (a + b)
fn add(a: &[u32], b: &[u32]) -> Vec<u32> {
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
fn sub(a: &[u32], b: &[u32]) -> Vec<u32> {
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
fn shift(a: &[u32], m: usize) -> Vec<u32> {
    let mut result = vec![0u32; m];
    result.extend_from_slice(a);
    result
}

/// からつば法の閾値: これ以下の桁数では筆算法にフォールバック
const KARATSUBA_THRESHOLD: usize = 32;

pub fn karatsuba(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (a, b) = pad(a, b);
    karatsuba_inner(&a, &b)
}

fn karatsuba_inner(a: &[u32], b: &[u32]) -> Vec<u32> {
    let n = a.len();

    if n <= KARATSUBA_THRESHOLD {
        return schoolbook(a, b);
    }

    let m = n / 2;

    // x = x1 * B^m + x0
    let (a0, a1) = a.split_at(m);
    let (b0, b1) = b.split_at(m);

    // p0 = a0 * b0
    let p0 = karatsuba_inner(a0, b0);
    // p2 = a1 * b1
    let p2 = karatsuba_inner(a1, b1);
    // p1 = (a0 + a1)(b0 + b1)
    let a_sum = add(a0, a1);
    let b_sum = add(b0, b1);
    let (a_sum, b_sum) = pad(&a_sum, &b_sum);
    let p1 = karatsuba_inner(&a_sum, &b_sum);

    // result = p2 * B^(2m) + (p1 - p2 - p0) * B^m + p0
    let middle = sub(&sub(&p1, &p2), &p0);

    let mut result = p0;
    let shifted_middle = shift(&middle, m);
    let shifted_p2 = shift(&p2, 2 * m);

    result = add(&result, &shifted_middle);
    result = add(&result, &shifted_p2);

    trim(&mut result);
    result
}

// =========================================================
// 3. Toom-3 — O(n^1.465)
// =========================================================

/// 符号付き多倍長整数
#[derive(Clone)]
struct Signed {
    digits: Vec<u32>,
    negative: bool,
}

impl Signed {
    fn from_digits(digits: Vec<u32>) -> Self {
        Self {
            digits,
            negative: false,
        }
    }

    fn zero() -> Self {
        Self {
            digits: vec![0],
            negative: false,
        }
    }

    fn is_zero(&self) -> bool {
        self.digits.iter().all(|&d| d == 0)
    }

    fn add(&self, other: &Signed) -> Signed {
        match (self.negative, other.negative) {
            (false, false) => Signed::from_digits(add(&self.digits, &other.digits)),
            (true, true) => {
                let mut r = Signed::from_digits(add(&self.digits, &other.digits));
                r.negative = true;
                r
            }
            (false, true) => self.sub_positive(other),
            (true, false) => other.sub_positive(self),
        }
    }

    fn sub(&self, other: &Signed) -> Signed {
        let mut neg = other.clone();
        neg.negative = !neg.negative;
        self.add(&neg)
    }

    /// self(positive) - other(treat as positive)
    fn sub_positive(&self, other: &Signed) -> Signed {
        if ge_abs(&self.digits, &other.digits) {
            Signed::from_digits(sub(&self.digits, &other.digits))
        } else {
            let mut r = Signed::from_digits(sub(&other.digits, &self.digits));
            r.negative = true;
            r
        }
    }

    fn mul(&self, other: &Signed) -> Signed {
        let digits = karatsuba(&self.digits, &other.digits);
        Signed {
            digits,
            negative: self.negative ^ other.negative,
        }
    }

    /// スカラー倍
    fn scale(&self, s: i64) -> Signed {
        let neg = if s < 0 {
            !self.negative
        } else {
            self.negative
        };
        let s = s.unsigned_abs();
        let mut result = Vec::with_capacity(self.digits.len() + 1);
        let mut carry: u64 = 0;
        for &d in &self.digits {
            let v = d as u64 * s + carry;
            result.push((v % BASE) as u32);
            carry = v / BASE;
        }
        if carry > 0 {
            result.push(carry as u32);
        }
        let mut r = Signed {
            digits: result,
            negative: neg,
        };
        trim(&mut r.digits);
        r
    }

    /// 正確に小整数で割る（余りなし前提）
    fn div_exact(&self, d: i64) -> Signed {
        let neg = if d < 0 {
            !self.negative
        } else {
            self.negative
        };
        let d = d.unsigned_abs();
        let mut result = vec![0u32; self.digits.len()];
        let mut rem: u64 = 0;
        for i in (0..self.digits.len()).rev() {
            let cur = rem * BASE + self.digits[i] as u64;
            result[i] = (cur / d) as u32;
            rem = cur % d;
        }
        let mut r = Signed {
            digits: result,
            negative: neg,
        };
        trim(&mut r.digits);
        r
    }

    fn to_unsigned(self) -> Vec<u32> {
        // 結果は非負のはず
        self.digits
    }
}

fn ge_abs(a: &[u32], b: &[u32]) -> bool {
    let a_len = a.len();
    let b_len = b.len();
    let a_eff = a.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    let b_eff = b.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    if a_eff != b_eff {
        return a_eff >= b_eff;
    }
    for i in (0..a_eff).rev() {
        let ai = if i < a_len { a[i] } else { 0 };
        let bi = if i < b_len { b[i] } else { 0 };
        if ai != bi {
            return ai > bi;
        }
    }
    true
}

fn shift_signed(s: &Signed, m: usize) -> Signed {
    Signed {
        digits: shift(&s.digits, m),
        negative: s.negative,
    }
}

const TOOM3_THRESHOLD: usize = 32;

pub fn toom3(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (a, b) = pad(a, b);
    let result = toom3_inner(&a, &b);
    result.to_unsigned()
}

fn toom3_inner(a: &[u32], b: &[u32]) -> Signed {
    let n = a.len();

    if n <= TOOM3_THRESHOLD {
        return Signed::from_digits(karatsuba(a, b));
    }

    let m = (n + 2) / 3;

    // 3分割: a = a2*B^(2m) + a1*B^m + a0
    let a0 = &a[..m.min(n)];
    let a1 = if m < n {
        &a[m..(2 * m).min(n)]
    } else {
        &[0u32] as &[u32]
    };
    let a2 = if 2 * m < n {
        &a[2 * m..]
    } else {
        &[0u32] as &[u32]
    };

    let b0 = &b[..m.min(n)];
    let b1 = if m < n {
        &b[m..(2 * m).min(n)]
    } else {
        &[0u32] as &[u32]
    };
    let b2 = if 2 * m < n {
        &b[2 * m..]
    } else {
        &[0u32] as &[u32]
    };

    let sa0 = Signed::from_digits(a0.to_vec());
    let sa1 = Signed::from_digits(a1.to_vec());
    let sa2 = Signed::from_digits(a2.to_vec());
    let sb0 = Signed::from_digits(b0.to_vec());
    let sb1 = Signed::from_digits(b1.to_vec());
    let sb2 = Signed::from_digits(b2.to_vec());

    // 評価点: 0, 1, -1, 2, ∞
    let fa_0 = sa0.clone();
    let fa_1 = sa0.add(&sa1).add(&sa2);
    let fa_m1 = sa0.sub(&sa1).add(&sa2);
    let fa_2 = sa0.add(&sa1.scale(2)).add(&sa2.scale(4));
    let fa_inf = sa2.clone();

    let fb_0 = sb0.clone();
    let fb_1 = sb0.add(&sb1).add(&sb2);
    let fb_m1 = sb0.sub(&sb1).add(&sb2);
    let fb_2 = sb0.add(&sb1.scale(2)).add(&sb2.scale(4));
    let fb_inf = sb2.clone();

    // 5回の乗算
    let w0 = fa_0.mul(&fb_0);
    let w1 = fa_1.mul(&fb_1);
    let wm1 = fa_m1.mul(&fb_m1);
    let w2 = fa_2.mul(&fb_2);
    let winf = fa_inf.mul(&fb_inf);

    // 補間 (GMP / Marco Bodrato の Toom-3 補間)
    // h(t) = r0 + r1*t + r2*t^2 + r3*t^3 + r4*t^4
    //
    // Given: w0=h(0), w1=h(1), wm1=h(-1), w2=h(2), winf=h(∞)
    //
    // Step 1: r0 = w0, r4 = winf
    // Step 2: r3 = (w2 - wm1) / 3
    // Step 3: r1 = (w1 - wm1) / 2
    // Step 4: r2 = w1 - w0
    // Step 5: r3 = (r3 - r2) / 2
    // Step 6: r2 = r2 - r1 - r4
    // Step 7: r3 = r3 - 2*r4
    // Step 8: r1 = r1 - r3

    let r0 = w0.clone();
    let r4 = winf.clone();

    let mut r3 = w2.sub(&wm1).div_exact(3);   // (w2 - wm1) / 3
    let mut r1 = w1.sub(&wm1).div_exact(2);   // (w1 - wm1) / 2
    let mut r2 = w1.sub(&w0);                  // w1 - w0

    r3 = r3.sub(&r2).div_exact(2);             // (r3 - r2) / 2
    r2 = r2.sub(&r1).sub(&r4);                 // r2 - r1 - r4
    r3 = r3.sub(&r4.scale(2));                 // r3 - 2*r4
    r1 = r1.sub(&r3);                          // r1 - r3

    // result = r4*B^(4m) + r3*B^(3m) + r2*B^(2m) + r1*B^m + r0
    let mut result = r0;
    result = result.add(&shift_signed(&r1, m));
    result = result.add(&shift_signed(&r2, 2 * m));
    result = result.add(&shift_signed(&r3, 3 * m));
    result = result.add(&shift_signed(&r4, 4 * m));

    let mut digits = result.to_unsigned();
    trim(&mut digits);
    Signed::from_digits(digits)
}

// =========================================================
// ユーティリティ
// =========================================================

/// ランダムな n 桁（BASE進）の数を生成
pub fn random_digits(n: usize) -> Vec<u32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut digits = Vec::with_capacity(n);
    for i in 0..n {
        let mut hasher = DefaultHasher::new();
        (i, n, 42u64).hash(&mut hasher);
        let h = hasher.finish();
        digits.push((h % BASE as u64) as u32);
    }
    // 最上位桁がゼロにならないように
    if let Some(last) = digits.last_mut() {
        if *last == 0 {
            *last = 1;
        }
    }
    digits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_u128(digits: &[u32]) -> u128 {
        let mut result: u128 = 0;
        let mut base_power: u128 = 1;
        for &d in digits {
            result += d as u128 * base_power;
            base_power *= BASE as u128;
        }
        result
    }

    #[test]
    fn test_schoolbook() {
        let a = vec![1234u32, 5678];
        let b = vec![8765u32, 4321];
        let result = schoolbook(&a, &b);
        assert_eq!(to_u128(&result), to_u128(&a) * to_u128(&b));
    }

    #[test]
    fn test_karatsuba() {
        let a = random_digits(100);
        let b = random_digits(100);
        let expected = schoolbook(&a, &b);
        let got = karatsuba(&a, &b);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_toom3() {
        let a = random_digits(200);
        let b = random_digits(200);
        let expected = schoolbook(&a, &b);
        let got = toom3(&a, &b);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_karatsuba_small() {
        // 1234 * 5678 = 7006652
        let a = vec![1234u32];
        let b = vec![5678u32];
        let result = karatsuba(&a, &b);
        assert_eq!(to_u128(&result), 1234 * 5678);
    }

    #[test]
    fn test_all_agree() {
        for size in [10, 50, 100, 200, 500] {
            let a = random_digits(size);
            let b = random_digits(size);
            let s = schoolbook(&a, &b);
            let k = karatsuba(&a, &b);
            let t = toom3(&a, &b);
            assert_eq!(s, k, "schoolbook != karatsuba at size {}", size);
            assert_eq!(s, t, "schoolbook != toom3 at size {}", size);
        }
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    fn test_toom3_small_debug() {
        // size=100 は閾値64を超えるのでtoom3が動く
        // まずsize=100で最小再現
        let a = random_digits(100);
        let b = random_digits(100);
        let expected = schoolbook(&a, &b);
        let got = toom3(&a, &b);
        
        // 最初に違う位置を見つける
        for i in 0..expected.len().max(got.len()) {
            let e = if i < expected.len() { expected[i] } else { 0 };
            let g = if i < got.len() { got[i] } else { 0 };
            if e != g {
                eprintln!("First diff at index {}: expected={}, got={}", i, e, g);
                break;
            }
        }
        
        // n=100, m=34 なので各部分は34桁
        // 結果は200桁くらい
        eprintln!("expected len={}, got len={}", expected.len(), got.len());
    }
    
    #[test]
    fn test_toom3_minimal() {
        // TOOM3_THRESHOLD=64なので、65で試す
        let a = random_digits(65);
        let b = random_digits(65);
        let expected = schoolbook(&a, &b);
        let got = toom3(&a, &b);
        assert_eq!(got, expected, "toom3 fails at size 65");
    }
}
