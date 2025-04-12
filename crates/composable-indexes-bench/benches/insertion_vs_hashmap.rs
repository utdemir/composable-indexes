use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn std_hashmap_insertion(n: u64) -> u64 {
    let mut map = std::collections::HashMap::new();
    for i in 0..n {
        map.insert(i, i);
    }
    map.len() as u64
}

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("insert");
    for i in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("std::collections::HashMap", i),
            i,
            |b, i| b.iter(|| black_box(std_hashmap_insertion(*i))),
        );

        group.bench_with_input(
            BenchmarkId::new("composable_indexes::Collection", i),
            i,
            |b, i| {
                b.iter_batched(
                    || composable_indexes::Collection::new(composable_indexes::index::trivial()),
                    |mut col| {
                        for j in 0..*i {
                            col.insert(j);
                        }
                        black_box(col.len() as u64);
                    },
                    BatchSize::SmallInput,
                )
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_insert);
criterion_main!(benches);
