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
    let mut db = Database::<Person, _>::new(premap(
        |p: &Person| p.name.clone(),
        indexes::btree::<String>(),
    ));

    db.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    db.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = db.query();
    let res = q.get(&"Alice".to_string());
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
    let mut db = Database::<Person, _>::new(zip2(
        premap(|p: &Person| p.name.clone(), indexes::btree::<String>()),
        premap(|p: &Person| p.age, indexes::btree::<u32>()),
    ));

    db.insert(Person {
        name: "Alice".to_string(),
        age: 42,
    });
    db.insert(Person {
        name: "Bob".to_string(),
        age: 24,
    });

    let q = db.query();
    let res = q.1.max();
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
