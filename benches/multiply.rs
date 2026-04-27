use bigint_mul_bench::{karatsuba, random_digits_with_seed, schoolbook, toom3};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_multiply(c: &mut Criterion) {
    let sizes = [10, 50, 100, 200, 500, 1000, 2000, 5000];

    let mut group = c.benchmark_group("multiply");

    for &size in &sizes {
        let a = random_digits_with_seed(size, 42);
        let b = random_digits_with_seed(size, 137);

        group.bench_with_input(BenchmarkId::new("schoolbook", size), &size, |bench, _| {
            bench.iter(|| schoolbook(&a, &b));
        });

        group.bench_with_input(BenchmarkId::new("karatsuba", size), &size, |bench, _| {
            bench.iter(|| karatsuba(&a, &b));
        });

        group.bench_with_input(BenchmarkId::new("toom3", size), &size, |bench, _| {
            bench.iter(|| toom3(&a, &b));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_multiply);
criterion_main!(benches);
