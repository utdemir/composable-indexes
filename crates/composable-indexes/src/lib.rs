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

pub use composable_indexes_core::Collection;

pub mod aggregation;
pub mod index;
