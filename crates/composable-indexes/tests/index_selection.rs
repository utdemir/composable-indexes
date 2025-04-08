use composable_indexes::{
    indexes::{premap, zip2},
    *,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    age: u32,
}

#[test]
fn ix1() {
    let mut collection = Collection::<Person, _>::new(premap(
        |p: &Person| p.name.clone(),
        indexes::btree::<String>(),
    ));

    collection.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    collection.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = collection.query();
    let res = q.get_one(&"Alice".to_string());
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
    let mut collection = Collection::<Person, _>::new(zip2(
        premap(|p: &Person| p.name.clone(), indexes::btree::<String>()),
        premap(|p: &Person| p.age, indexes::btree::<u32>()),
    ));

    collection.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    collection.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = collection.query();
    let res = q.1.max_one();
    assert_eq!(
        res,
        Some((
            &42,
            &Person {
                name: "Alice".to_string(),
                age: 42
            }
        ))
    );
}
