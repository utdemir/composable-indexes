//! Module providing various index implementations and combinators.
//! Includes basic indexes like BTree and HashTable, as well as
//! combinators for transforming, grouping, and filtering indexes.

pub mod btree;

#[doc(hidden)]
pub use btree::btree;

pub mod premap;

#[doc(hidden)]
pub use premap::premap;

pub mod grouped;

#[doc(hidden)]
pub use grouped::grouped;

pub mod trivial;

#[doc(hidden)]
pub use trivial::trivial;

pub mod filtered;

#[doc(hidden)]
pub use filtered::filtered;

pub mod hashtable;

#[doc(hidden)]
pub use hashtable::hashtable;

pub mod zip;
pub use composable_indexes_derive::zip;
