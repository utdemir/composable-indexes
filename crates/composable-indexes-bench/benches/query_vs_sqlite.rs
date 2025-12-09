use composable_indexes::{Collection, aggregation, index};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sqlite::{Connection, State};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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

    conn
}

fn criterion_benchmark(c: &mut Criterion) {
    let data = get_test_data(10000);

    let sqlite_conn = setup_sqlite(&data);

    let mut composable_db = Collection::new(index::zip!(
        index::premap(|p: &Person| p.name.clone(), index::hashtable()),
        index::premap(|p: &Person| p.birth_year, index::btree()),
        index::grouped(|p: &Person| p.starsign.clone(), || aggregation::count()),
    ));

    for person in &data {
        composable_db.insert(person.clone());
    }

    // Benchmark looking up by name
    c.bench_function("lookup_by_name_sqlite", |b| {
        b.iter(|| {
            let mut stmt = sqlite_conn
                .prepare("SELECT * FROM people WHERE name = ?")
                .unwrap();
            stmt.bind((1, "Person_5000")).unwrap();
            stmt.next().unwrap();
        })
    });

    c.bench_function("lookup_by_name_composable", |b| {
        b.iter(|| {
            black_box(
                composable_db.execute(|ix| ix._1().inner().get_one(&"Person_5000".to_string())),
            );
        })
    });

    // Benchmark finding max birth year
    c.bench_function("max_birth_year_sqlite", |b| {
        b.iter(|| {
            let mut stmt = sqlite_conn
                .prepare("SELECT MAX(birth_year) FROM people")
                .unwrap();
            stmt.next().unwrap();
        })
    });

    c.bench_function("max_birth_year_composable", |b| {
        b.iter(|| {
            black_box(composable_db.execute(|ix| ix._2().inner().max_one()));
        })
    });

    // Benchmark counting by starsign
    c.bench_function("count_by_starsign_sqlite", |b| {
        b.iter(|| {
            let mut stmt = sqlite_conn
                .prepare("SELECT starsign, COUNT(*) FROM people GROUP BY starsign")
                .unwrap();
            while stmt.next().unwrap() != State::Done {}
        })
    });

    c.bench_function("count_by_starsign_composable", |b| {
        b.iter(|| {
            black_box(
                composable_db.execute(|ix| ix._3().get(&StarSign::Gemini).map(|ix| ix.get())),
            );
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
