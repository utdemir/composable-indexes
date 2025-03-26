use composable_indexes::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Person {
    name: String,
    age: u32,
}

#[test]
fn ix0() {
    let mut db = Database::<Person>::empty();

    db.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    let bob = db.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = db.query();
    let res = q.0.get(&bob);
    assert_eq!(
        res,
        Some(&Person {
            name: "Bob".to_string(),
            age: 24
        })
    );
}

#[test]
fn ix1() {
    let mut db = Database::<Person>::empty()
        .register_index(premap(|p: &Person| p.name.clone(), indexes::btree()));

    db.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    db.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = db.query();
    let res = q.1.get(&"Alice".to_string());
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
    let mut db = Database::<Person>::empty()
        .register_index(premap(|p: &Person| p.name.clone(), indexes::btree()))
        .register_index(premap(|p: &Person| p.age, indexes::btree()));

    db.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    db.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = db.query();
    let res = q.2.max();
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
