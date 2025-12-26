use composable_indexes::{Collection, index};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rusqlite::{Connection, params};
use std::collections::HashMap;

// Generate sample data - random ASCII strings
fn generate_strings(n: usize) -> Vec<String> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hash, Hasher};

    let hasher_builder = RandomState::new();
    (0..n)
        .map(|i| {
            let mut hasher = hasher_builder.build_hasher();
            i.hash(&mut hasher);
            let seed = hasher.finish();

            // Generate a random string of length 20-40
            let len = 20 + (seed % 21) as usize;
            let mut s = String::with_capacity(len);
            let mut rng = seed;
            for _ in 0..len {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let c = (b'a' + ((rng >> 16) % 26) as u8) as char;
                s.push(c);
            }
            s
        })
        .collect()
}

// Reference implementation: just a Vec with manual filtering
struct VecStringStore {
    data: HashMap<u64, String>,
    next_key: u64,
}

impl VecStringStore {
    fn new() -> Self {
        VecStringStore {
            data: HashMap::new(),
            next_key: 0,
        }
    }

    fn insert(&mut self, value: String) -> u64 {
        let key = self.next_key;
        self.next_key += 1;
        self.data.insert(key, value);
        key
    }

    fn contains_get_all(&self, pattern: &str) -> Vec<u64> {
        self.data
            .iter()
            .filter(|(_, v)| v.contains(pattern))
            .map(|(k, _)| *k)
            .collect()
    }
}

// SQLite implementation with LIKE query
struct SqliteStringStore {
    conn: Connection,
}

impl SqliteStringStore {
    fn with_strings(strings: &[String]) -> Self {
        let conn = Connection::open_in_memory().unwrap();

        // Create table with index on the text column
        conn.execute(
            "CREATE TABLE strings (id INTEGER PRIMARY KEY, text TEXT NOT NULL)",
            [],
        )
        .unwrap();

        conn.execute("CREATE INDEX idx_strings_text ON strings(text)", [])
            .unwrap();

        // Insert all strings
        let mut stmt = conn
            .prepare("INSERT INTO strings (id, text) VALUES (?, ?)")
            .unwrap();
        for (i, s) in strings.iter().enumerate() {
            stmt.execute(params![i as i64, s]).unwrap();
        }
        drop(stmt);

        SqliteStringStore { conn }
    }

    fn contains_get_all(&self, pattern: &str) -> Vec<u64> {
        let query = format!("%{}%", pattern);
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM strings WHERE text LIKE ?")
            .unwrap();
        let mut rows = stmt.query(params![query]).unwrap();

        let mut keys = Vec::new();
        while let Ok(Some(row)) = rows.next() {
            let id: i64 = row.get(0).unwrap();
            keys.push(id as u64);
        }
        keys
    }
}

// Benchmark contains_get_all with different approaches and sample sizes
fn bench_stringsearch_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("stringsearch/contains");

    for n in [5, 50, 100, 200, 500, 1000, 2000, 4000, 6000, 10000, 15000].iter() {
        let strings = generate_strings(*n);

        // Benchmark suffix_tree index
        let mut col_suffix = Collection::new(index::suffix_tree());
        for s in &strings {
            col_suffix.insert(s.clone());
        }
        group.bench_with_input(BenchmarkId::new("suffix_tree", n), n, |b, _| {
            b.iter(|| {
                let result = col_suffix.query(|ix| ix.contains_get_all("abcd"));
                black_box(result)
            });
        });

        // Benchmark naive Vec implementation
        let mut vec_store = VecStringStore::new();
        for s in &strings {
            vec_store.insert(s.clone());
        }
        group.bench_with_input(BenchmarkId::new("vec_filter", n), n, |b, _| {
            b.iter(|| {
                let result = vec_store.contains_get_all("abcd");
                black_box(result)
            });
        });

        // Benchmark SQLite implementation
        let sqlite_store = SqliteStringStore::with_strings(&strings);
        group.bench_with_input(BenchmarkId::new("sqlite", n), n, |b, _| {
            b.iter(|| {
                let result = sqlite_store.contains_get_all("abcd");
                black_box(result)
            });
        });
    }

    group.finish();
}

// Benchmark insertion performance with string indexing
fn bench_stringsearch_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("stringsearch/insert");

    for n in [5, 50, 100, 200, 500, 1000, 2000, 4000, 6000, 10000, 15000].iter() {
        let strings = generate_strings(*n);

        group.bench_with_input(
            BenchmarkId::new("suffix_tree", n),
            &strings,
            |b, strings| {
                b.iter(|| {
                    let mut col = Collection::new(index::suffix_tree());
                    for s in strings {
                        col.insert(s.clone());
                    }
                    black_box(col.len())
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("vec_store", n), &strings, |b, strings| {
            b.iter(|| {
                let mut store = VecStringStore::new();
                for s in strings {
                    store.insert(s.clone());
                }
                black_box(store.data.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("sqlite", n), &strings, |b, strings| {
            b.iter(|| {
                let _store = SqliteStringStore::with_strings(strings);
                black_box(1) // just measuring construction time
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_stringsearch_contains,
    bench_stringsearch_insert
);
criterion_main!(benches);
