use bigint_mul_bench::{karatsuba, random_digits_with_seed, schoolbook, toom3};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let sizes = [10, 50, 100, 200, 500, 1000, 2000, 5000];

    println!(
        "{:>8} | {:>14} | {:>14} | {:>14} | {:>10} | {:>10}",
        "digits", "schoolbook", "karatsuba", "toom3", "K/S ratio", "T/S ratio"
    );
    println!("{}", "-".repeat(88));

    for &size in &sizes {
        // a と b に異なるシードを使い、同一値にならないようにする
        let a = random_digits_with_seed(size, 42);
        let b = random_digits_with_seed(size, 137);

        // 3つのアルゴリズムの結果が一致することを確認（不一致ならバグ）
        let s = schoolbook(&a, &b);
        let k = karatsuba(&a, &b);
        let t = toom3(&a, &b);
        assert_eq!(s, k, "schoolbook != karatsuba at size {}", size);
        assert_eq!(s, t, "schoolbook != toom3 at size {}", size);

        // ウォームアップを兼ねる
        black_box(&s);
        black_box(&k);
        black_box(&t);

        let iters = match size {
            n if n <= 100 => 1000,
            n if n <= 500 => 100,
            n if n <= 2000 => 10,
            _ => 5,
        };

        // black_box で結果を包み、コンパイラによるデッドコード除去を防ぐ
        let start = Instant::now();
        for _ in 0..iters {
            black_box(schoolbook(black_box(&a), black_box(&b)));
        }
        let school_time = start.elapsed().as_nanos() as f64 / iters as f64;

        let start = Instant::now();
        for _ in 0..iters {
            black_box(karatsuba(black_box(&a), black_box(&b)));
        }
        let kara_time = start.elapsed().as_nanos() as f64 / iters as f64;

        let start = Instant::now();
        for _ in 0..iters {
            black_box(toom3(black_box(&a), black_box(&b)));
        }
        let toom_time = start.elapsed().as_nanos() as f64 / iters as f64;

        fn format_time(ns: f64) -> String {
            if ns < 1_000.0 {
                format!("{:.0} ns", ns)
            } else if ns < 1_000_000.0 {
                format!("{:.1} µs", ns / 1_000.0)
            } else if ns < 1_000_000_000.0 {
                format!("{:.2} ms", ns / 1_000_000.0)
            } else {
                format!("{:.2} s", ns / 1_000_000_000.0)
            }
        }

        println!(
            "{:>8} | {:>14} | {:>14} | {:>14} | {:>9.2}x | {:>9.2}x",
            size,
            format_time(school_time),
            format_time(kara_time),
            format_time(toom_time),
            school_time / kara_time,
            school_time / toom_time,
        );
    }
}
