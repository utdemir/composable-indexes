use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn std_hashmap_insertion(n: u64) -> u64 {
    let mut map = std::collections::HashMap::new();
    for i in 0..n {
        map.insert(i, i);
    }
    map.len() as u64
}

fn composable_indexes_insertion(n: u64) -> u64 {
    let mut db = composable_indexes::Database::new(composable_indexes::indexes::trivial());
    for i in 0..n {
        db.insert(i);
    }
    db.len() as u64
}

fn bench_fibs(c: &mut Criterion) {
    let mut group = c.benchmark_group("insertion");
    for i in [10_000, 20_000, 30_000, 40_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("std::collections::HashMap", i),
            &i,
            |b, i| b.iter(|| black_box(std_hashmap_insertion(**i))),
        );
        group.bench_with_input(BenchmarkId::new("composable_indexes::DB", i), &i, |b, i| {
            b.iter(|| black_box(composable_indexes_insertion(**i)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);
