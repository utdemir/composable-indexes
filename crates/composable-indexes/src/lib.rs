//! A library for in-memory collections with simple, performant and composable indexes.
//!
//! # Example
//!
//! ```rust
//! use composable_indexes::{Collection, index, aggregation};
//!
//! struct Person { name: String, age: u32, occupation: String }
//!
//! // Define indexes on the Person struct
//! #[derive(composable_indexes::Index)]
//! #[index(Person)]
//! struct PersonIndex {
//!   by_name: index::PremapIndex<Person, String, index::HashTableIndex<String>>,
//!   by_age: index::PremapOwnedIndex<Person, u32, index::BTreeIndex<u32>>,
//!   by_occupation: index::GroupedIndex<Person, String, aggregation::CountIndex>,
//! }
//!
//! // Create the collection.
//! let mut collection = Collection::new(
//!   PersonIndex {
//!     by_name: index::premap(|p: &Person| &p.name, index::hashtable()),
//!     by_age: index::premap_owned(|p: &Person| p.age, index::btree()),
//!     by_occupation: index::grouped(|p: &Person| &p.occupation, || aggregation::count()),
//!   }
//! );
//!
//! // insert items
//! collection.insert(Person { name: "Alice".to_string(), age: 30, occupation: "Engineer".to_string() });
//! collection.insert(Person { name: "Bob".to_string(), age: 25, occupation: "Designer".to_string() });
//!
//! // Query by name - returns keys, use get_one to get the first match
//! let alice_key = collection.query(|ix| ix.by_name.get_one(&"Alice".to_string()));
//!
//! // Query oldest person
//! let _oldest = collection.query(|ix| ix.by_age.max_one());
//!
//! // Query the count of designers
//! let _designer_count = collection.query(|ix| ix.by_occupation.get(&"Designer".to_string()).get());
//! ```
//!
//! # Motivation
//!
//! Rust standard library (and ecosystem) provides excellent in-memory data structures like `HashMap`,
//! `BTreeMap`, `Vec`, and so on. However, if we need to index multiple fields of a data structure,
//! we will need to build and maintain multiple data structures, and carefully write code to keep
//! them in sync.
//!
//! `composable-indexes` aims to solve this problem by introducing a concept called [`Index`](core::Index)es, which
//! are composable data structures that are declared upfront and automatically kept in sync with the
//! main collection. It's designed to be lightweight, easy to extend, and efficient.
//!
//! ## Collection
//!
//! At the core of `composable-indexes` is the [`Collection<T, Ix>`](Collection) type, which represents
//! a collection of items of type `T`, indexed by a set of indexes `Ix`. A [`Collection`] owns the data,
//! and the indexes hold pointers to the data in the collection. When we insert, update, or delete items
//! in the collection, it automatically updates the indexes accordingly.
//!
//! When inserting an item into a [`Collection`](Collection), it returns a [`Key`](Key) which can be used to refer to
//! the inserted item later (for updates or deletions) without resorting to a query.
//!
//! **Warning**: The collection can only guarantee the validity of the index as long as the items are not
//! mutated in place. This should only be possible using interior mutability (e.g., `Cell`, `RefCell`,
//! `Mutex`, etc.). So - do not use interior mutability on the fields that are indexed.
//!
//! But the more interesting part is querying. Methods like `query`, `update` and `delete` methods accept
//! a callback that receives a reference to the index structure, allowing us to perform efficient lookups
//! using the indexes defined on the collection.
//!
//! # Performance
//!
//! `composable-indexes` is designed with performance in mind. The interfaces are designed
//! to compile away, and only expose the underlying data structures. In other words, think
//! of a [`Collection`] as a way to translate operations to the underlying index structures
//! at compile time without adding (significant) runtime overhead.
//!
//! Data structures in `composable-indexes` hold the data entirely in memory. Usually, the
//! data is owned by the [`Collection`] itself, and indexes hold pointers to the data in the
//! collection (called a [`Key`]). This means that for most queries you can expect one
//! lookup to the index structure to obtain the pointer, and then one lookup to the
//! collection to obtain the actual data.
//!
//! ## Index Performance
//!
//! The common indexes ([`btree`](index::btree), [`hashtable`](index::hashtable)) are simply thin wrappers around
//! `std::collections::BTreeMap` and `std::collections::HashMap`, so you can expect the
//! same performance characteristics as those data structures. They are keyed by the input
//! (usually a field of the stored type) and values are sets of pointers to the actual
//! data stored in the collection.
//!
//! Higher order indexes like [`filtered`](index::filtered), [`premap`](index::premap) are all zero-cost abstractions and have
//! negligible overhead.
//!
//! **Important**: Because of not doing bookkeeping themselves, the functions passed to
//! higher-order indexes should be fast to compute, as they will not be cached and are
//! computed on-the-fly. Ideally, they should be things like field accesses rather than
//! expensive computations.
//!
//! The most commonly used indexes are [`hashtable`](index::hashtable) for equality lookups and [`btree`](index::btree) for
//! range queries. Between those two, hashtables are the fastest. They also come with
//! immutable counterparts (with the `imbl` feature) which tend to be slower, but allow
//! cheap cloning and multi-versioning of the database.
//!
//! | Index Type | Operations | Insert | Remove | Query | Memory |
//! |------------|------------|--------|--------|-------|--------|
//! | [`KeysIndex`](index::KeysIndex) | get all keys | O(1) | O(1) | O(n) | O(n) |
//! | [`HashTableIndex`](index::HashTableIndex) | contains, get, count distinct | O(1) | O(1) | O(1) | O(n) |
//! | [`BTreeIndex`](index::BTreeIndex) | contains, get, count distinct, range, min, max | O(log n) | O(log n) | O(log n) | O(n) |
//! | [`SuffixTreeIndex`](index::SuffixTreeIndex) | string search | O(k * log n) † | O(k * log n) † | O(log n) | O(n) ‡ |
//! | Aggregations | count, sum, mean, min, max | O(1) | O(1) | O(1) | O(1) |
//!
//! † k = length of the string
//!
//! ‡ Suffix trees have have a high memory footprint (even though linear), expect 5-10 times the length of the input strings.
//!
//!
//! ## Aggregation Performance
//!
//! All built-in aggregations are calculated iteratively, without holding the data in
//! memory. You can expect O(1) memory and time complexity regardless of the size of the
//! collection.
//!
//! As an example, [`aggregations::count`](aggregation::count) simply increments and decrements a counter as
//! items are inserted and removed, [`aggregations::mean`](aggregation::mean) only keeps track of the sum and
//! count and so on.
//!
//! ## Indexing Overhead
//!
//! A [`Collection`] is simply a `HashMap`, and indexes are additional data structures.
//! Hence, inserting an element into a [`Collection`] simply compiles down to inserting
//! the element into the underlying `HashMap`, and then inserting pointers to the same
//! element into each of the indexes. Hence, the overhead of adding indexes is linear
//! in the number of indexes.
//!
//! As an example, in the benchmark below we compare inserting elements into a
//! `std::collections::HashMap` versus inserting the same elements into a
//! `composable_indexes::Collection` with zero, one, two, three, and four indexes.
//! You can see that without an index, the performance is exactly the same as a
//! `HashMap`, and adding an index linearly increases the insertion time.
//!
#![doc=include_str!("../doc_assets/bench_indexing_overhead.svg")]
//!
//! # Security
//!
//! As both [`Collection`] and [`hashtable`](index::hashtable) index are backed by hash maps, the choice of the
//! hash function can have a significant impact on performance. `composable-indexes`
//! defaults to the default hasher of `hashbrown`, which is `foldhash` that is fast,
//! but not cryptographically secure or prone to HashDoS attacks. If you need a different
//! hasher, they can be provided when constructing collection and the indexes. See `foldhash`'s
//! README for more details: <https://github.com/orlp/foldhash>
//!

#![cfg_attr(all(not(feature = "std"), not(feature = "testutils")), no_std)]

extern crate alloc;

pub mod core;
pub use core::{Collection, Key, ShallowClone};

pub mod aggregation;
pub mod index;

#[cfg(feature = "testutils")]
pub mod testutils;

#[cfg(feature = "derive")]
pub use composable_indexes_derive::{Index, ShallowClone};

// Some tests for the Collection functionality are defined
// here so we can utilize the testutils crate.
#[cfg(test)]
mod test {
    use crate::core::*;
    use crate::testutils::test_index;

    macro_rules! op_insert {
        ($key:expr, $new:expr) => {
            $crate::testutils::Op::Insert($crate::testutils::Insert_ {
                key: Key { id: $key },
                new: $new,
            })
        };
    }

    macro_rules! op_update {
        ($key:expr, $existing:expr, $new:expr) => {
            $crate::testutils::Op::Update($crate::testutils::Update_ {
                key: Key { id: $key },
                new: $new,
                existing: $existing,
            })
        };
    }

    macro_rules! op_remove {
        ($key:expr, $existing:expr) => {
            $crate::testutils::Op::Remove($crate::testutils::Remove_ {
                key: Key { id: $key },
                existing: $existing,
            })
        };
    }

    #[test]
    fn simple() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        let two = db.insert(2);
        let three = db.insert(3);
        db.update_by_key(two, |_| 10);
        let four = db.insert(4);
        db.delete_by_key(three);

        assert_eq!(db.get_by_key(one), Some(&1));
        assert_eq!(db.get_by_key(two), Some(&10));
        assert_eq!(db.get_by_key(three), None);
        assert_eq!(db.get_by_key(four), Some(&4));
        assert_eq!(db.len(), 3);

        // Access test index operations directly
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
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
        db.update_by_key_mut(one, |v| {
            if let Some(v) = v {
                *v += 1;
            }
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn update_mut_inserts() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete_by_key(one);
        db.update_by_key_mut(one, |v| {
            assert!(v.is_none());
            *v = Some(2);
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn update_mut_removes() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.update_by_key_mut(one, |v| {
            assert!(v.is_some());
            *v = None;
        });

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![op_insert!(0, 1), op_remove!(0, 1),]);
    }

    #[test]
    fn update_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.update_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![op_insert!(0, 1), op_update!(0, 1, 2),]);
    }

    #[test]
    fn update_inserts() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.update_by_key(one, |x| {
            assert_eq!(x, None);
            2
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.adjust_by_key_mut(one, |v| {
            *v = 2;
        });

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(
            ops,
            vec![op_insert!(0, 1), op_remove!(0, 1), op_insert!(0, 2),]
        );
    }

    #[test]
    fn adjust_mut_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.adjust_by_key_mut(one, |_| {
            panic!("Should not be called");
        });

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![op_insert!(0, 1), op_remove!(0, 1),]);
    }

    #[test]
    fn adjust_updates() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.adjust_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), Some(&2));
        assert_eq!(db.len(), 1);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![op_insert!(0, 1), op_update!(0, 1, 2),]);
    }

    #[test]
    fn adjust_ignores_non_existent() {
        let mut db = Collection::<u32, _>::new(test_index());

        let one = db.insert(1);
        db.delete_by_key(one);

        db.adjust_by_key(one, |_| 2);

        assert_eq!(db.get_by_key(one), None);
        assert_eq!(db.len(), 0);
        let ops = db.query(|ix| Plain(ix.ops.clone()));
        assert_eq!(ops, vec![op_insert!(0, 1), op_remove!(0, 1),]);
    }
}
