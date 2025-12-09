use divan::{Bencher, black_box};

fn main() {
    divan::main();
}

fn std_hashmap_insertion(n: u64) -> u64 {
    let mut map = std::collections::HashMap::new();
    for i in 0..n {
        map.insert(i, i);
    }
    map.len() as u64
}

#[divan::bench(args = [10, 100, 1000, 10000])]
fn hashmap(n: u64) -> u64 {
    black_box(std_hashmap_insertion(n))
}

#[divan::bench(args = [10, 100, 1000, 10000])]
fn collection(bencher: Bencher, n: u64) {
    bencher
        .with_inputs(|| composable_indexes::Collection::new(composable_indexes::index::trivial()))
        .bench_values(|mut col| {
            for j in 0..n {
                col.insert(j);
            }
            black_box(col.len() as u64)
        });
}
