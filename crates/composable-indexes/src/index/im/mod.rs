//! Immutable persistent index implementations using the `imbl` library.
//! These indexes provide persistent data structures that allow for efficient
//! structural sharing and immutability.

pub mod btree;

#[doc(hidden)]
pub use btree::{BTreeIndex, btree};

pub mod hashtable;

#[doc(hidden)]
pub use hashtable::{HashTableIndex, hashtable};

#[doc(hidden)]
pub mod grouped;

pub use grouped::{GroupedIndex, grouped};

#[doc(hidden)]
pub mod keys;

pub use keys::{KeysIndex, keys};