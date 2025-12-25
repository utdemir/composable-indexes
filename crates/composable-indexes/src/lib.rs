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
//! # Quick Start
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! // A struct representing a person
//! struct Person { name: String, age: u32, occupation: String }
//!
//! // Create a collection indexed by age using premap
//! let mut collection = Collection::<Person, _>::new(
//!   index::zip!(
//!     index::premap_owned(|p: &Person| p.age, index::btree()),
//!     index::premap_owned(|p: &Person| p.occupation.clone(), index::btree()),
//!   )
//! );
//!
//! // insert & update collection
//! let alice = collection.insert(Person { name: "Alice".to_string(), age: 30, occupation: "Engineer".to_string() });
//! collection.insert(Person { name: "Bob".to_string(), age: 25, occupation: "Designer".to_string() });
//! collection.adjust_by_key_mut(alice, |p| { p.age = 31; });
//! // ...
//!
//! // Query oldest person
//! let _oldest = collection.query(|ix| ix._1().max_one());
//!
//! // Query the number of unique occupations
//! let _occupation_count = collection.query(|ix| ix._2().count_distinct());
//! ```
//!
//! # Motivation
//!
//! Rust standard library (and ecosystem) provides excellent in-memory data structures like `HashMap`,
//! `BTreeMap`, `Vec`, and so on. However, if we need to index multiple fields of a data structure,
//! we will need to build and maintain multiple data structures, and carefully write code to keep
//! them in sync.
//!
//! `composable-indexes` aims to solve this problem by introducing a concept called `Index`es, which
//! are composable data structures that are declared upfront and automatically kept in sync with the
//! main collection. It's designed to be lightweight, easy to extend, and efficient.
//!
//! ## Collection
//!
//! At the core of `composable-indexes` is the [`Collection<T, Ix>`](Collection) type, which represents
//! a collection of items of type `T`, indexed by a set of indexes `Ix`. A `Collection` owns the data,
//! and the indexes hold pointers to the data in the collection.
//!
//! # Available Indexes
//!
//! ## Basic Indexes
//!
//! ### TrivialIndex
//!
//! The simplest possible index—it does nothing. Constructor: [`trivial()`](index::trivial).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//! let mut collection = Collection::<u32, _>::new(index::trivial());
//! ```
//!
//! ### KeysIndex
//!
//! Maintains the set of all keys in the collection. Constructor: [`keys()`](index::keys).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! let mut collection = Collection::<String, _>::new(index::keys());
//! let key1 = collection.insert("hello".to_string());
//! let key2 = collection.insert("world".to_string());
//!
//! // Iterate over all keys
//! let all_keys = collection.query(|ix| ix.all().collect::<Vec<_>>());
//! ```
//!
//! ### HashTableIndex
//!
//! An index backed by a hash map for fast equality lookups. Constructor: [`hashtable()`](index::hashtable).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! let mut collection = Collection::<String, _>::new(index::hashtable());
//! let key = collection.insert("hello".to_string());
//!
//! // Fast O(1) lookup
//! let items = collection.query(|ix| ix.get(&"hello".to_string()));
//! ```
//!
//! **Performance**: O(1) average-case insertion/removal/lookup
//!
//! ### BTreeIndex
//!
//! An index backed by a B-tree for ordered access and range queries. Constructor: [`btree()`](index::btree).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! let mut collection = Collection::<i32, _>::new(index::btree());
//! collection.insert(42);
//! collection.insert(10);
//! collection.insert(100);
//!
//! // Range query
//! let items = collection.query(|ix| ix.range(20..50));
//!
//! // Min/max
//! let min = collection.query(|ix| ix.min_one());
//! let max = collection.query(|ix| ix.max_one());
//! ```
//!
//! **Performance**: O(log n) insertion/removal/lookup/range queries
//!
//! ## Index Combinators
//!
//! ### PremapIndex / PremapOwnedIndex
//!
//! Transforms items before passing them to an inner index. Constructors: [`premap()`](index::premap),
//! [`premap_owned()`](index::premap_owned).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! // Index by age (owned value)
//! let by_age = index::premap_owned(
//!     |p: &Person| p.age,
//!     index::btree()
//! );
//!
//! let mut people = Collection::<Person, _>::new(by_age);
//! people.insert(Person { name: "Alice".to_string(), age: 30 });
//!
//! // Query people aged 25-35
//! let adults = people.query(|ix| ix.range(25..35));
//! ```
//!
//! **Performance**: Zero-cost abstraction—no runtime overhead beyond the inner index
//!
//! **Important**: The transformation function is called every time, so keep it fast
//! (ideally just field access).
//!
//! ### GroupedIndex
//!
//! Groups items by a key and maintains a separate index for each group. Similar to SQL's
//! GROUP BY. Constructor: [`grouped()`](index::grouped).
//!
//! ```rust
//! use composable_indexes::{Collection, index, aggregation};
//!
//! struct Task {
//!     project: String,
//!     status: String,
//! }
//!
//! // Group by project, track all tasks
//! let by_project = index::grouped(
//!     |t: &Task| t.project.clone(),
//!     || index::keys()
//! );
//!
//! let mut tasks = Collection::<Task, _>::new(by_project);
//! tasks.insert(Task {
//!     project: "WebApp".to_string(),
//!     status: "done".to_string(),
//! });
//!
//! // Get all tasks in "WebApp" project
//! let webapp_tasks = tasks.query(|ix|
//!     ix.get(&"WebApp".to_string()).all()
//! );
//! ```
//!
//! **Use cases**: One-to-many relationships, aggregating data by category, nested grouping
//!
//! ### FilteredIndex
//!
//! Only indexes items that match a predicate. Constructor: [`filtered()`](index::filtered).
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! struct Person {
//!     name: String,
//!     age: u32,
//! }
//!
//! // Index only adults
//! let adult_index = index::filtered(
//!     |p: &Person| if p.age >= 18 { Some(p.age) } else { None },
//!     index::btree()
//! );
//!
//! let mut people = Collection::<Person, _>::new(adult_index);
//! people.insert(Person { name: "Alice".to_string(), age: 30 });
//! people.insert(Person { name: "Bob".to_string(), age: 15 });
//!
//! // Only sees adults
//! let adults = people.query(|ix| ix.range(0..));  // Only Alice
//! ```
//!
//! ### ZipIndex
//!
//! Combines multiple indexes into one. Constructor: [`zip!`](index::zip!) macro.
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! struct Person {
//!     name: String,
//!     age: u32,
//!     email: String,
//! }
//!
//! let person_index = index::zip!(
//!     index::premap(|p: &Person| &p.name, index::hashtable()),
//!     index::premap_owned(|p: &Person| p.age, index::btree()),
//! );
//!
//! let mut people = Collection::<Person, _>::new(person_index);
//! people.insert(Person {
//!     name: "Alice".to_string(),
//!     age: 30,
//!     email: "alice@example.com".to_string(),
//! });
//!
//! // Query by name
//! let by_name = people.query(|ix| ix._1().get(&"Alice".to_string()));
//!
//! // Query by age range
//! let by_age = people.query(|ix| ix._2().range(25..35));
//! ```
//!
//! Access each index using `._1()`, `._2()`, `._3()`, etc. Available up to `ZipIndex10`.
//!
//! ## Aggregation Indexes
//!
//! Aggregation indexes compute statistics over collections. They maintain running totals
//! that update incrementally.
//!
//! ### CountIndex
//!
//! Counts the number of items. Constructor: [`count()`](aggregation::count).
//!
//! ```rust
//! use composable_indexes::{Collection, aggregation};
//!
//! let mut collection = Collection::<String, _>::new(aggregation::count());
//! collection.insert("a".to_string());
//! collection.insert("b".to_string());
//!
//! let count = collection.query(|ix| ix.get());  // 2
//! ```
//!
//! **Performance**: O(1) for all operations
//!
//! ### SumIndex
//!
//! Sums numeric values. Constructor: [`sum()`](aggregation::sum).
//!
//! ```rust
//! use composable_indexes::{Collection, aggregation};
//!
//! let mut scores = Collection::<i32, _>::new(aggregation::sum());
//! scores.insert(10);
//! scores.insert(20);
//! scores.insert(30);
//!
//! let total = scores.query(|ix| ix.get());  // 60
//! ```
//!
//! ### MeanIndex
//!
//! Computes the arithmetic mean. Constructor: [`mean()`](aggregation::mean).
//!
//! ```rust
//! use composable_indexes::{Collection, aggregation};
//!
//! let mut scores = Collection::<i32, _>::new(aggregation::mean());
//! scores.insert(10);
//! scores.insert(20);
//! scores.insert(30);
//!
//! let average = scores.query(|ix| ix.get());  // 20.0
//! ```
//!
//! ### MinIndex / MaxIndex
//!
//! Finds the minimum or maximum value. Constructors: [`min()`](aggregation::min),
//! [`max()`](aggregation::max).
//!
//! ```rust
//! use composable_indexes::{Collection, aggregation};
//!
//! let mut temps = Collection::<i32, _>::new(aggregation::max());
//! temps.insert(20);
//! temps.insert(25);
//! temps.insert(18);
//!
//! let highest = temps.query(|ix| ix.get());  // Some(25)
//! ```
//!
//! **Performance**: O(log n) for updates (maintains a BTree internally)
//!
//! ## Persistent (Immutable) Indexes
//!
//! With the `imbl` feature enabled, you get persistent (immutable) versions of indexes
//! using structural sharing from the `imbl` crate.
//!
//! Available under `index::im::*`: `hashtable()`, `btree()`, `keys()`, `grouped()`
//!
//! ```rust,ignore
//! use composable_indexes::{Collection, index};
//!
//! let mut db = Collection::<String, _>::new(index::im::hashtable());
//! db.insert("value1".to_string());
//!
//! // Clone is cheap - shares structure
//! let db_snapshot = db.shallow_clone();
//!
//! db.insert("value2".to_string());
//! // Original snapshot unchanged
//! ```
//!
//! **Benefits**: Cheap cloning via structural sharing, multi-versioning of collections
//!
//! **Tradeoffs**: Slower than mutable versions, higher memory overhead
//!
//! # Performance
//!
//! `composable-indexes` is designed with performance in mind. The interfaces are designed
//! to compile away, and only expose the underlying data structures. In other words, think
//! of a `Collection` as a way to translate operations to the underlying index structures
//! at compile time without adding (significant) runtime overhead.
//!
//! Data structures in `composable-indexes` hold the data entirely in memory. Usually, the
//! data is owned by the `Collection` itself, and indexes hold pointers to the data in the
//! collection (called a [`Key`]). This means that for most queries you can expect one
//! lookup to the index structure to obtain the pointer, and then one lookup to the
//! collection to obtain the actual data.
//!
//! ## Index Performance
//!
//! The common indexes (`btree`, `hashtable`) are simply thin wrappers around
//! `std::collections::BTreeMap` and `std::collections::HashMap`, so you can expect the
//! same performance characteristics as those data structures. They are keyed by the input
//! (usually a field of the stored type) and values are sets of pointers to the actual
//! data stored in the collection.
//!
//! Higher order indexes like `filtered`, `premap` are all zero-cost abstractions and have
//! negligible overhead.
//!
//! **Important**: Because of not doing bookkeeping themselves, the functions passed to
//! higher-order indexes should be fast to compute, as they will not be cached and are
//! computed on-the-fly. Ideally, they should be things like field accesses rather than
//! expensive computations.
//!
//! The most commonly used indexes are `hashtable` for equality lookups and `btree` for
//! range queries. Between those two, hashtables are the fastest. They also come with
//! immutable counterparts (with the `imbl` feature) which tend to be slower, but allow
//! cheap cloning and multi-versioning of the database.
//!
//! As both `Collection` and `hashtable` index are backed by hash maps, the choice of the
//! hash function can have a significant impact on performance. `composable-indexes`
//! defaults to the default hasher (from `std` if the `std` feature is enabled, from
//! `hashbrown` otherwise), but can be overridden.
//!
//! ## Aggregation Performance
//!
//! All built-in aggregations are calculated iteratively, without holding the data in
//! memory. You can expect O(1) memory and time complexity regardless of the size of the
//! collection.
//!
//! As an example, `aggregations::count` simply increments and decrements a counter as
//! items are inserted and removed, `aggregations::mean` only keeps track of the sum and
//! count and so on.
//!
//! ## Indexing Overhead
//!
//! A `Collection` is simply a `HashMap`, and indexes are additional data structures.
//! Hence, inserting an element into a `Collection` simply compiles down to inserting
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
//!
//! ## Performance Characteristics Table
//!
//! | Index Type | Insert | Remove | Query | Memory |
//! |------------|--------|--------|-------|--------|
//! | `TrivialIndex` | O(1) | O(1) | N/A | O(1) |
//! | `KeysIndex` | O(1) | O(1) | O(n) | O(n) |
//! | `HashTableIndex` | O(1)* | O(1)* | O(1)* | O(n) |
//! | `BTreeIndex` | O(log n) | O(log n) | O(log n) | O(n) |
//! | `PremapIndex` | Inner + O(1) | Inner + O(1) | Inner | O(1) |
//! | `GroupedIndex` | Inner + O(1) | Inner + O(1) | Inner | O(n) |
//! | `FilteredIndex` | Inner | Inner | Inner | O(1) |
//! | `ZipIndex` | Sum of inners | Sum of inners | O(1) per index | Sum of inners |
//! | Aggregations | O(1)† | O(1)† | O(1) | O(1) |
//!
//! \* Average case; worst case O(n)  
//! † Except `min()`/`max()` which are O(log n)

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
