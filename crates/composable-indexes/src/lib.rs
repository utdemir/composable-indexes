//! A library for in-memory collections with flexible and composable indexes.
//!
//! This crate provides a framework for building collections with multiple indexes
//! that can be combined and composed. Key features include:
//!
//! - Built-in indexes for common use cases (BTree, HashTable)
//! - Composable index combinators (grouping, filtering, mapping)
//! - Aggregation indexes for statistical operations
//! - Safe and efficient index maintenance as collection changes
//!
//! # Example
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! // A struct representing a person
//! struct Person { name: String, age: u32, occupation: String }
//!
//! // Create a collection indexed by age using premap
//! let mut collection = Collection::<Person, _>::new(
//!   index::zip!(
//!     index::premap(|p: &Person| p.age, index::btree()),
//!     index::premap(|p: &Person| p.occupation.clone(), index::hashtable()),
//!   )
//! );
//!
//! // insert & update collection
//! let alice = collection.insert(Person { name: "Alice".to_string(), age: 30, occupation: "Engineer".to_string() });
//! collection.insert(Person { name: "Bob".to_string(), age: 25, occupation: "Designer".to_string() });
//! collection.adjust_mut(alice, |p| { p.age = 31; });
//! // ...
//!
//! let q = collection.query();
//!
//! // Query oldest person
//! let _youngest = q.0.max_one();
//!
//! // Query the number of unique occupations
//! let _occupation_count = q.1.count_distinct();

pub use composable_indexes_core::{Collection, Key};

pub mod aggregation;
pub mod index;

// Some tests for the Collection functionality is defined
// here so we can utilise the testutils crate.
#[cfg(test)]
mod test {
    use composable_indexes_core::*;
    use composable_indexes_testutils::{op_insert, op_remove, op_update, test_index};

    #[test]
    fn simple() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        let two = db.insert(2);
        let three = db.insert(3);
        db.update(two, |_| 10);
        let four = db.insert(4);
        db.delete(&three);

        assert_eq!(db.get(one), Some(&1));
        assert_eq!(db.get(two), Some(&10));
        assert_eq!(db.get(three), None);
        assert_eq!(db.get(four), Some(&4));
        assert_eq!(db.len(), 3);

        let q = db.query();
        assert_eq!(
            q.operations(),
            vec![
                op_insert!(0, 1),
                op_insert!(1, 2),
                op_insert!(2, 3),
                op_update!(1, 2, 10),
                op_insert!(3, 4),
                op_remove!(2, 3),
            ]
        );
    }

    #[test]
    fn update_mut_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.update_mut(one, |v| {
            if let Some(v) = v {
                *v += 1;
            }
        });

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn update_mut_inserts() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete(&one);
        db.update_mut(one, |v| {
            assert!(v.is_none());
            *v = Some(2);
        });

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn update_mut_removes() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.update_mut(one, |v| {
            assert!(v.is_some());
            *v = None;
        });

        assert_eq!(db.get(one), None);
        assert_eq!(db.len(), 0);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1),]
        );
    }

    #[test]
    fn update_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.update(one, |_| 2);

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_update!(0, 1, 2),]
        );
    }

    #[test]
    fn update_inserts() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete(&one);

        db.update(one, |x| {
            assert_eq!(x, None);
            2
        });

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.adjust_mut(one, |v| {
            *v = 2;
        });

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete(&one);

        db.adjust_mut(one, |_| {
            panic!("Should not be called");
        });

        assert_eq!(db.get(one), None);
        assert_eq!(db.len(), 0);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1),]
        );
    }

    #[test]
    fn adjust_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.adjust(one, |_| 2);

        assert_eq!(db.get(one), Some(&2));
        assert_eq!(db.len(), 1);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_update!(0, 1, 2),]
        );
    }

    #[test]
    fn adjust_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete(&one);

        db.adjust(one, |_| 2);

        assert_eq!(db.get(one), None);
        assert_eq!(db.len(), 0);
        assert_eq!(
            db.query().operations(),
            vec![op_insert!(0, 1), op_remove!(0, 1),]
        );
    }
}
