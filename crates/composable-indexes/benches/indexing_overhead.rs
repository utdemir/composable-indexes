use composable_indexes::{Collection, index};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashMap;

// Benchmark comparing insertion overhead with increasing number of hashtable indexes
fn bench_indexing_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexing_overhead");

    for n in [100, 200, 300, 400, 500, 750, 1000, 2000, 5000, 10000].iter() {
        // Baseline: raw HashMap
        group.bench_with_input(
            BenchmarkId::new("hashmap_with_default_hasher", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for i in 0..n {
                        map.insert(i as u64, i as u64);
                    }
                    black_box(map.len())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hashmap_with_foldhash_hasher", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut map: foldhash::HashMap<u64, u64> = foldhash::HashMap::default();
                    for i in 0..n {
                        map.insert(i as u64, i as u64);
                    }
                    black_box(map.len())
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("collection_with_no_index", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut col = Collection::new(index::Trivial);
                    for i in 0..n {
                        col.insert(i as u64);
                    }
                    black_box(col.len())
                });
            },
        );

        // Collection with 1 hashtable index
        group.bench_with_input(
            BenchmarkId::new("collection_with_1_index", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut col = Collection::new(index::HashTable::<u64>::new());
                    for i in 0..n {
                        col.insert(i as u64);
                    }
                    black_box(col.len())
                });
            },
        );

        // Collection with 2 hashtable indexes
        group.bench_with_input(
            BenchmarkId::new("collection_with_2_indexes", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut col = Collection::new((
                        index::HashTable::<u64>::new(),
                        index::PremapOwned::new(|x: &u64| x * 2, index::HashTable::<u64>::new()),
                    ));
                    for i in 0..n {
                        col.insert(i as u64);
                    }
                    black_box(col.len())
                });
            },
        );

        // Collection with 3 hashtable indexes
        group.bench_with_input(
            BenchmarkId::new("collection_with_3_indexes", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut col = Collection::new((
                        index::HashTable::<u64>::new(),
                        index::PremapOwned::new(|x: &u64| x * 2, index::HashTable::<u64>::new()),
                        index::PremapOwned::new(|x: &u64| x * 3, index::HashTable::<u64>::new()),
                    ));
                    for i in 0..n {
                        col.insert(i as u64);
                    }
                    black_box(col.len())
                });
            },
        );

        // Collection with 4 hashtable indexes
        group.bench_with_input(
            BenchmarkId::new("collection_with_4_indexes", n),
            n,
            |b, &n| {
                b.iter(|| {
                    let mut col = Collection::new((
                        index::HashTable::<u64>::new(),
                        index::PremapOwned::new(|x: &u64| x * 2, index::HashTable::<u64>::new()),
                        index::PremapOwned::new(|x: &u64| x * 3, index::HashTable::<u64>::new()),
                        index::PremapOwned::new(|x: &u64| x * 4, index::HashTable::<u64>::new()),
                    ));
                    for i in 0..n {
                        col.insert(i as u64);
                    }
                    black_box(col.len())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_indexing_overhead);
criterion_main!(benches);
