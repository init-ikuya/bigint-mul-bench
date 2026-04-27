use crate::common::{trim, BASE};

/// 筆算法 (Schoolbook) — O(n²)
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

    let mut out = Vec::with_capacity(result.len());
    let mut carry: u64 = 0;
    for &v in &result {
        let val = v + carry;
        out.push((val % BASE) as u32);
        carry = val / BASE;
    }
    if carry > 0 {
        out.push(carry as u32);
    }
    trim(&mut out);
    out
}
