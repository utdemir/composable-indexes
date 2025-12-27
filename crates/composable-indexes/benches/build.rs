use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");

    for n in [100, 200, 300, 400, 500, 750, 1000, 2000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("hashmap", n), n, |b, &n| {
            b.iter(|| {
                let mut map = std::collections::HashMap::<u64, u64>::new();
                for i in 0..n {
                    map.insert(i, i);
                }
                black_box(map.len() as u64)
            });
        });

        group.bench_with_input(BenchmarkId::new("collection", n), n, |b, &n| {
            b.iter(|| {
                let mut col = composable_indexes::Collection::new(());
                for j in 0..n {
                    col.insert(j);
                }
                black_box(col.len() as u64)
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_build);
criterion_main!(benches);
