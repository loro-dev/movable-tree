use criterion::{criterion_main, Criterion};
use movable_tree::Forest;

pub fn benches() {
    let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args().sample_size(10);
    let mut group = criterion.benchmark_group("Apply n inserts");
    group.bench_function("n = 10K", |b| {
        b.iter(|| {
            let mut forest: Forest<usize> = Forest::new();
            forest.mov(0, None).unwrap();
            for i in 0..10_000 {
                forest.mov(i + 1, Some(i)).unwrap();
            }
        });
    });

    group.bench_function("n = 100K", |b| {
        b.iter(|| {
            let mut forest: Forest<usize> = Forest::new();
            forest.mov(0, None).unwrap();
            for i in 0..100_000 {
                forest.mov(i + 1, Some(i)).unwrap();
            }
        });
    });

    group.bench_function("n = 1M", |b| {
        b.iter(|| {
            let mut forest: Forest<usize> = Forest::new();
            forest.mov(0, None).unwrap();
            for i in 0..1_000_000 {
                forest.mov(i + 1, Some(i)).unwrap();
            }
        });
    });
}

criterion_main!(benches);
