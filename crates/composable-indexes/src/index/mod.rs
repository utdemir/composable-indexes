//! Module providing various index implementations and combinators.
//! Includes basic indexes like BTree and HashTable, as well as
//! combinators for transforming, grouping, and filtering indexes.

pub mod generic;

mod btree;
pub use btree::BTree;

mod premap;
pub use premap::{GenericPremap, Premap, PremapOwned};

mod grouped;
pub use grouped::{GenericGrouped, Grouped, GroupedOwned};

mod trivial;
pub use trivial::Trivial;

mod filtered;
pub use filtered::Filtered;

mod hashtable;
pub use hashtable::HashTable;

mod keys;
pub use keys::Keys;

mod suffix_tree;
pub use suffix_tree::SuffixTree;

mod zip;

#[cfg(feature = "imbl")]
pub mod im;
