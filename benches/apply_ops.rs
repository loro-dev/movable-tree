use std::time::Instant;

use criterion::{criterion_main, Criterion};
use movable_tree::{crdt::Crdt, Forest};
use rand::{rngs::StdRng, Rng};

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
    drop(group);

    let mut group = criterion.benchmark_group("Random n moves in tree with 10K nodes");
    group.sample_size(10);
    let SIZE: usize = 10_000;
    group.bench_function("n = 10K", |b| {
        const N: usize = 10_000;
        let mut source_forest: Forest<usize> = Forest::new();
        source_forest.mov(0, None).unwrap();
        for i in 0..SIZE {
            source_forest.mov(i + 1, Some(0)).unwrap();
        }

        b.iter_custom(|iters| {
            let start = Instant::now();
            for _ in 0..iters {
                let mut forest = source_forest.clone();
                let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);
                for _ in 0..N {
                    let i = rng.gen::<usize>() % SIZE;
                    let j = rng.gen::<usize>() % SIZE;
                    forest.mov(i, Some(j)).unwrap_or_default();
                }
            }
            start.elapsed()
        });
    });

    drop(group);
    let mut group = criterion.benchmark_group("CRDT n moves with 10K nodes");
    group.sample_size(10);
    bench_crdt(&mut group, "n = 10K", SIZE, 10_000);
    bench_crdt(&mut group, "n = 100K", SIZE, 100_000);
    bench_crdt(&mut group, "n = 1M", SIZE, 1_000_000);
}

fn bench_crdt(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    size: usize,
    n: usize,
) {
    group.bench_function(name, |b| {
        let mut crdt: Crdt = Crdt::new(1);
        let mut ids = Vec::new();
        for _ in 0..size {
            ids.push(crdt.new_node(None));
        }

        b.iter_batched(
            || crdt.clone(),
            |mut crdt: Crdt| {
                let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);
                for _ in 0..n {
                    let i = rng.gen::<usize>() % size;
                    let j = rng.gen::<usize>() % size;
                    crdt.mov(ids[i], Some(ids[j]));
                }
            },
            criterion::BatchSize::PerIteration,
        );
    });
}

criterion_main!(benches);
