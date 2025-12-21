use composable_indexes::{Collection, index};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// Benchmark insert performance - all index types in one group
fn bench_indexes_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexes/insert");

    for n in [100, 200, 300, 400, 500, 750, 1000, 2000, 5000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("hashtable", n), n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::hashtable::<u64>());
                for i in 0..n {
                    col.insert(i as u64);
                }
                black_box(col.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("btree", n), n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::btree::<u64>());
                for i in 0..n {
                    col.insert(i as u64);
                }
                black_box(col.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("im::hashtable", n), n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::im::hashtable::<u64>());
                for i in 0..n {
                    col.insert(i as u64);
                }
                black_box(col.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("im::btree", n), n, |b, &n| {
            b.iter(|| {
                let mut col = Collection::new(index::im::btree::<u64>());
                for i in 0..n {
                    col.insert(i as u64);
                }
                black_box(col.len())
            });
        });
    }

    group.finish();
}

// Benchmark get_one performance - all index types in one group
fn bench_indexes_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("indexes/get");

    for n in [100, 200, 300, 400, 500, 750, 1000, 2000, 5000, 10000].iter() {
        let mut col_hashtable = Collection::new(index::hashtable::<u64>());
        for i in 0..*n {
            col_hashtable.insert(i as u64);
        }
        group.bench_with_input(BenchmarkId::new("hashtable", n), n, |b, &n| {
            b.iter(|| black_box(col_hashtable.query(|ix| ix.get_one(&((n / 2) as u64)))));
        });

        let mut col_btree = Collection::new(index::btree::<u64>());
        for i in 0..*n {
            col_btree.insert(i as u64);
        }
        group.bench_with_input(BenchmarkId::new("btree", n), n, |b, &n| {
            b.iter(|| black_box(col_btree.query(|ix| ix.get_one(&((n / 2) as u64)))));
        });

        let mut col_im_hashtable = Collection::new(index::im::hashtable::<u64>());
        for i in 0..*n {
            col_im_hashtable.insert(i as u64);
        }
        group.bench_with_input(BenchmarkId::new("im::hashtable", n), n, |b, &n| {
            b.iter(|| black_box(col_im_hashtable.query(|ix| ix.get_one(&((n / 2) as u64)))));
        });

        let mut col_im_btree = Collection::new(index::im::btree::<u64>());
        for i in 0..*n {
            col_im_btree.insert(i as u64);
        }
        group.bench_with_input(BenchmarkId::new("im::btree", n), n, |b, &n| {
            b.iter(|| black_box(col_im_btree.query(|ix| ix.get_one(&((n / 2) as u64)))));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_indexes_insert, bench_indexes_get);
criterion_main!(benches);
