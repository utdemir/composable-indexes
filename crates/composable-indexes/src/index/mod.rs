//! Module providing various index implementations and combinators.
//! Includes basic indexes like BTree and HashTable, as well as
//! combinators for transforming, grouping, and filtering indexes.

pub mod btree;

#[doc(hidden)]
pub use btree::{BTreeIndex, btree};

pub mod premap;

#[doc(hidden)]
pub use premap::{PremapIndex, premap};

pub mod grouped;

#[doc(hidden)]
pub use grouped::{GroupedIndex, grouped};

pub mod trivial;

#[doc(hidden)]
pub use trivial::{TrivialIndex, trivial};

pub mod filtered;

#[doc(hidden)]
pub use filtered::{FilteredIndex, filtered};

pub mod hashtable;

#[doc(hidden)]
pub use hashtable::{HashTableIndex, hashtable};

pub mod keys;

#[doc(hidden)]
pub use keys::{KeysIndex, keys};

pub mod zip;

#[doc(hidden)]
pub use zip::{
    ZipIndex2, ZipIndex3, ZipIndex4, ZipIndex5, ZipIndex6, ZipIndex7, ZipIndex8, ZipIndex9,
    ZipIndex10, ZipIndex11, ZipIndex12, ZipIndex13, ZipIndex14, ZipIndex15, ZipIndex16, zip,
};
