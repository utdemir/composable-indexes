use composable_indexes::{Collection, Key, index};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn bench_keysets_with_overlap(c: &mut Criterion, overlap_count: usize) {
    let mut group = c.benchmark_group(format!("Keysets with {} overlap", overlap_count));

    for n in [100, 200, 300, 400, 500, 750, 1000, 2000, 5000, 10000] {
        let mod_ = if overlap_count > 0 && n > overlap_count {
            n / overlap_count
        } else {
            n
        };

        group.bench_with_input(BenchmarkId::new("HashSet", n), &n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::HashTable::<
                    u64,
                    hashbrown::DefaultHashBuilder,
                    hashbrown::HashSet<Key>,
                >::new());

                for i in 0..n {
                    col.insert((i % mod_) as u64);
                }

                black_box(col.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("BTreeSet", n), &n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::HashTable::<
                    u64,
                    hashbrown::DefaultHashBuilder,
                    std::collections::BTreeSet<Key>,
                >::new());
                for i in 0..n {
                    col.insert((i % mod_) as u64);
                }
                black_box(col.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("RoaringTreemap", n), &n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::HashTable::<
                    u64,
                    hashbrown::DefaultHashBuilder,
                    roaring::RoaringTreemap,
                >::new());
                for i in 0..n {
                    col.insert((i % mod_) as u64);
                }
                black_box(col.len())
            });
        });
    }

    group.finish();
}

// Benchmark insert performance - all index types in one group
fn keysets_no_overlap(c: &mut Criterion) {
    bench_keysets_with_overlap(c, 0);
}

fn keysets_three_overlap(c: &mut Criterion) {
    bench_keysets_with_overlap(c, 3);
}

fn keysets_twelve_overlap(c: &mut Criterion) {
    bench_keysets_with_overlap(c, 12);
}

fn keysets_hundred_overlap(c: &mut Criterion) {
    bench_keysets_with_overlap(c, 100);
}

fn keysets_thousand_overlap(c: &mut Criterion) {
    bench_keysets_with_overlap(c, 1000);
}

criterion_group!(
    benches,
    keysets_no_overlap,
    keysets_three_overlap,
    keysets_twelve_overlap,
    keysets_hundred_overlap,
    keysets_thousand_overlap
);
criterion_main!(benches);
