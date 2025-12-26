use composable_indexes::{Collection, index};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    age: u32,
}

#[test]
fn ix1() {
    let mut collection = Collection::<Person, _>::new(index::PremapOwnedIndex::new(
        |p: &Person| p.name.clone(),
        index::btree::<String>(),
    ));

    collection.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    collection.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let res = collection.query(|ix| ix.get_one(&"Alice".to_string()));
    assert_eq!(
        res,
        Some(&Person {
            name: "Alice".to_string(),
            age: 42
        })
    );
}

#[test]
fn ix2() {
    let mut collection = Collection::<Person, _>::new(index::zip!(
        index::PremapOwnedIndex::new(|p: &Person| p.name.clone(), index::btree::<String>()),
        index::PremapOwnedIndex::new(|p: &Person| p.age, index::btree::<u32>()),
    ));

    collection.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    collection.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let res = collection.query(|ix| ix._2().max_one());
    assert_eq!(
        res,
        Some(&Person {
            name: "Alice".to_string(),
            age: 42
        })
    );
}
