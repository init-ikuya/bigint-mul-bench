use crate::common::{add, pad, shift, sub, trim};
use crate::schoolbook::schoolbook;

/// からつば法の閾値: これ以下の桁数では筆算法にフォールバック
const KARATSUBA_THRESHOLD: usize = 32;

/// からつば法 (Karatsuba) — O(n^1.585)
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
