use crate::common::{add, pad, shift, sub, trim, BASE};
use crate::karatsuba::karatsuba;
use crate::schoolbook::schoolbook;

/// 符号付き多倍長整数（Toom-3の補間で必要）
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
        let negative = self.negative ^ other.negative;
        Signed {
            negative: if digits == [0] { false } else { negative },
            digits,
        }
    }

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
        if r.digits == [0] { r.negative = false; }
        r
    }

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
        if r.digits == [0] { r.negative = false; }
        r
    }

    fn into_unsigned(self) -> Vec<u32> {
        self.digits
    }
}

fn ge_abs(a: &[u32], b: &[u32]) -> bool {
    let a_eff = a.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    let b_eff = b.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
    if a_eff != b_eff {
        return a_eff >= b_eff;
    }
    for i in (0..a_eff).rev() {
        let ai = if i < a.len() { a[i] } else { 0 };
        let bi = if i < b.len() { b[i] } else { 0 };
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

/// Toom-3 — O(n^1.465)
pub fn toom3(a: &[u32], b: &[u32]) -> Vec<u32> {
    let (a, b) = pad(a, b);
    let result = toom3_inner(&a, &b);
    result.into_unsigned()
}

fn toom3_inner(a: &[u32], b: &[u32]) -> Signed {
    let n = a.len();

    // 閾値以下では筆算法にフォールバック
    // karatsuba() 経由だと pad() による余分なコピーが発生するため直接 schoolbook を呼ぶ
    if n <= TOOM3_THRESHOLD {
        return Signed::from_digits(schoolbook(a, b));
    }

    let m = n.div_ceil(3);

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
    let r0 = w0.clone();
    let r4 = winf.clone();

    let mut r3 = w2.sub(&wm1).div_exact(3);
    let mut r1 = w1.sub(&wm1).div_exact(2);
    let mut r2 = w1.sub(&w0);

    r3 = r3.sub(&r2).div_exact(2);
    r2 = r2.sub(&r1).sub(&r4);
    r3 = r3.sub(&r4.scale(2));
    r1 = r1.sub(&r3);

    // result = r4*B^(4m) + r3*B^(3m) + r2*B^(2m) + r1*B^m + r0
    let mut result = r0;
    result = result.add(&shift_signed(&r1, m));
    result = result.add(&shift_signed(&r2, 2 * m));
    result = result.add(&shift_signed(&r3, 3 * m));
    result = result.add(&shift_signed(&r4, 4 * m));

    let mut digits = result.into_unsigned();
    trim(&mut digits);
    Signed::from_digits(digits)
}
