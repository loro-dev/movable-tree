use criterion::{criterion_main, Criterion};
use movable_tree::Forest;
pub fn benches() {
    let mut criterion: Criterion<_> = (Criterion::default()).configure_from_args().sample_size(10);
    let mut group = criterion.benchmark_group("preserve all history");
    group.bench_function(
        "insert 10K elements and preserving all history (then drop the history)",
        |b| {
            // It takes 1 ms to apply 10K ops
            // It takes 8 ms to apply and record 10K ops
            // It takes 9 ms to drop the history of 10K ops
            // It takes 19 ms in total
            b.iter(|| {
                let mut forest: Forest<usize> = Forest::new();
                let mut history = vec![];
                forest.mov(0, None).unwrap();
                for i in 0..10_000 {
                    history.push(forest.clone());
                    forest.mov(i + 1, Some(i)).unwrap();
                }
                // Dropping is slow

                // spawn(move || {
                //     drop(history);
                // });
            });
        },
    );

    group.bench_function(
        "insert 100K elements and preserving all history (then drop the history)",
        |b| {
            // It takes 337 ms in total
            // It takes 140 ms without dropping
            b.iter(|| {
                let mut forest: Forest<usize> = Forest::new();
                let mut history = vec![];
                forest.mov(0, None).unwrap();
                for i in 0..100_000 {
                    history.push(forest.clone());
                    forest.mov(i + 1, Some(i)).unwrap();
                }

                // Dropping is slow
                // spawn(move || {
                //     drop(history);
                // });
            });
        },
    );

    // criterion.bench_function(
    //     "insert 1M elements and preserving all history (then drop the history)",
    //     |b| {
    //         // It takes 337 ms in total
    //         // It takes 140 ms without dropping
    //         b.iter(|| {
    //             let mut forest: Forest<usize> = Forest::new();
    //             let mut history = vec![];
    //             forest.mov(0, None).unwrap();
    //             for i in 0..1_000_000 {
    //                 history.push(forest.clone());
    //                 forest.mov(i + 1, Some(i)).unwrap();
    //             }

    //             // Dropping is slow
    //             mem::forget(history);
    //         });
    //     },
    // );
}

criterion_main!(benches);
