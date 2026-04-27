pub mod common;
pub mod schoolbook;
pub mod karatsuba;
pub mod toom3;

pub use common::random_digits_with_seed;
pub use schoolbook::schoolbook;
pub use karatsuba::karatsuba;
pub use toom3::toom3;

#[cfg(test)]
mod tests {
    use super::*;
    use common::BASE;

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
        let a = random_digits_with_seed(100, 42);
        let b = random_digits_with_seed(100, 137);
        let expected = schoolbook(&a, &b);
        let got = karatsuba(&a, &b);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_toom3() {
        let a = random_digits_with_seed(200, 42);
        let b = random_digits_with_seed(200, 137);
        let expected = schoolbook(&a, &b);
        let got = toom3(&a, &b);
        assert_eq!(got, expected);
    }

    #[test]
    fn test_karatsuba_small() {
        let a = vec![1234u32];
        let b = vec![5678u32];
        let result = karatsuba(&a, &b);
        assert_eq!(to_u128(&result), 1234 * 5678);
    }

    #[test]
    fn test_all_agree() {
        for size in [10, 50, 100, 200, 500] {
            let a = random_digits_with_seed(size, 42);
            let b = random_digits_with_seed(size, 137);
            let s = schoolbook(&a, &b);
            let k = karatsuba(&a, &b);
            let t = toom3(&a, &b);
            assert_eq!(s, k, "schoolbook != karatsuba at size {}", size);
            assert_eq!(s, t, "schoolbook != toom3 at size {}", size);
        }
    }
}
