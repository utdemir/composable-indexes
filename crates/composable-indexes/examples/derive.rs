#![allow(dead_code)]

use composable_indexes::{Collection, aggregation, index};
use composable_indexes_derive::Index;

struct Person {
    name: String,
    age: u32,
    ssn: String,
}

#[derive(Index)]
#[index(Person)]
struct PersonIndex {
    by_age: index::PremapIndex<Person, u32, index::BTreeIndex<u32>>,
    by_ssn: index::PremapIndex<Person, String, index::HashTableIndex<String>>,
    count: aggregation::CountIndex,
}

impl PersonIndex {
    fn new() -> Self {
        Self {
            by_age: index::premap(|p: &Person| p.age, index::btree()),
            by_ssn: index::premap(|p: &Person| p.ssn.clone(), index::hashtable()),
            count: aggregation::count(),
        }
    }
}

fn main() {
    let mut db = Collection::<Person, PersonIndex>::new(PersonIndex::new());

    db.insert(Person {
        name: "Alice".to_string(),
        age: 30,
        ssn: "123-45-6789".to_string(),
    });

    db.insert(Person {
        name: "Bob".to_string(),
        age: 25,
        ssn: "987-65-4321".to_string(),
    });

    db.insert(Person {
        name: "Charlie".to_string(),
        age: 35,
        ssn: "555-55-5555".to_string(),
    });

    let oldest = db.query(|ix| ix.by_age.max_one());
    assert_eq!(oldest.unwrap().age, 35);

    let person = db.query(|ix| ix.by_ssn.get_one(&"123-45-6789".to_string()));
    assert_eq!(person.unwrap().name, "Alice");

    let total = db.query(|ix| ix.count.get());
    assert_eq!(total, 3);
}
