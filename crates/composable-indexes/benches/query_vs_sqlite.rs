use composable_indexes::{Collection, aggregation, index};
use divan::{Bencher, black_box};
use sqlite::{Connection, State};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum StarSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    birth_year: u16,
    starsign: StarSign,
}

impl Person {
    fn new(name: String, birth_year: u16, starsign: StarSign) -> Self {
        Self {
            name,
            birth_year,
            starsign,
        }
    }
}

fn get_test_data(count: usize) -> Vec<Person> {
    let starsigns = [
        StarSign::Aries,
        StarSign::Taurus,
        StarSign::Gemini,
        StarSign::Cancer,
    ];
    (0..count)
        .map(|i| {
            Person::new(
                format!("Person_{}", i),
                1950 + (i % 50) as u16,
                starsigns[i % 4].clone(),
            )
        })
        .collect()
}

fn setup_sqlite(data: &[Person]) -> Connection {
    let conn = Connection::open(":memory:").unwrap();

    // Optimize SQLite for in-memory performance
    conn.execute("PRAGMA synchronous = OFF").unwrap();
    conn.execute("PRAGMA journal_mode = MEMORY").unwrap();
    conn.execute("PRAGMA temp_store = MEMORY").unwrap();
    conn.execute("PRAGMA locking_mode = EXCLUSIVE").unwrap();

    conn.execute(
        "CREATE TABLE people (
            name TEXT PRIMARY KEY,
            birth_year INTEGER,
            starsign TEXT
        )",
    )
    .unwrap();

    conn.execute("CREATE INDEX idx_birth_year ON people(birth_year)")
        .unwrap();
    conn.execute("CREATE INDEX idx_starsign ON people(starsign)")
        .unwrap();

    {
        let mut stmt = conn
            .prepare("INSERT INTO people (name, birth_year, starsign) VALUES (?, ?, ?)")
            .unwrap();

        for person in data {
            stmt.bind((1, person.name.as_str())).unwrap();
            stmt.bind((2, person.birth_year as i64)).unwrap();
            stmt.bind((3, format!("{:?}", person.starsign).as_str()))
                .unwrap();
            stmt.next().unwrap();
            stmt.reset().unwrap();
        }
    }

    // Gather statistics for query planner
    conn.execute("ANALYZE").unwrap();

    conn
}

fn main() {
    divan::main();
}

// Note: Both SQLite and composable indexes are set up once outside the benchmark loop.
// For SQLite, we use stmt.reset() before each iteration.
// This ensures we're only measuring query execution time, not setup overhead.

#[divan::bench]
fn lookup_by_name_sqlite(bencher: Bencher) {
    let data = get_test_data(100_000);
    let conn = setup_sqlite(&data);
    let mut stmt = conn.prepare("SELECT * FROM people WHERE name = ?").unwrap();

    bencher.bench_local(|| {
        stmt.reset().unwrap();
        stmt.bind((1, "Person_5000")).unwrap();
        stmt.next().unwrap();
    });
}

#[divan::bench]
fn lookup_by_name_composable(bencher: Bencher) {
    let data = get_test_data(100_000);
    let mut db = Collection::new(index::zip!(
        index::premap(|p: &Person| p.name.clone(), index::hashtable()),
        index::premap(|p: &Person| p.birth_year, index::btree()),
        index::grouped(
            |p: &Person| p.starsign.clone(),
            || aggregation::count::<u32>()
        ),
    ));
    for person in &data {
        db.insert(person.clone());
    }

    bencher.bench_local(|| {
        black_box(db.query(|ix| ix._1().get_one(&"Person_5000".to_string())));
    });
}

#[divan::bench]
fn max_birth_year_sqlite(bencher: Bencher) {
    let data = get_test_data(100_000);
    let conn = setup_sqlite(&data);
    let mut stmt = conn.prepare("SELECT MAX(birth_year) FROM people").unwrap();

    bencher.bench_local(|| {
        stmt.reset().unwrap();
        stmt.next().unwrap();
    });
}

#[divan::bench]
fn max_birth_year_composable(bencher: Bencher) {
    let data = get_test_data(100_000);
    let mut db = Collection::new(index::zip!(
        index::premap(|p: &Person| p.name.clone(), index::hashtable()),
        index::premap(|p: &Person| p.birth_year, index::btree()),
        index::grouped(
            |p: &Person| p.starsign.clone(),
            || aggregation::count::<u32>()
        ),
    ));
    for person in &data {
        db.insert(person.clone());
    }

    bencher.bench_local(|| {
        black_box(db.query(|ix| ix._2().max_one()));
    });
}

#[divan::bench]
fn count_by_starsign_sqlite(bencher: Bencher) {
    let data = get_test_data(100_000);
    let conn = setup_sqlite(&data);
    let mut stmt = conn
        .prepare("SELECT starsign, COUNT(*) FROM people GROUP BY starsign")
        .unwrap();

    bencher.bench_local(|| {
        stmt.reset().unwrap();
        while stmt.next().unwrap() != State::Done {}
    });
}

#[divan::bench]
fn count_by_starsign_composable(bencher: Bencher) {
    let data = get_test_data(100_000);
    let mut db = Collection::new(index::zip!(
        index::premap(|p: &Person| p.name.clone(), index::hashtable()),
        index::premap(|p: &Person| p.birth_year, index::btree()),
        index::grouped(
            |p: &Person| p.starsign.clone(),
            || aggregation::count::<u32>()
        ),
    ));
    for person in &data {
        db.insert(person.clone());
    }

    bencher.bench_local(|| {
        black_box(db.query(|ix| ix._3().get(&StarSign::Gemini).get()));
    });
}
