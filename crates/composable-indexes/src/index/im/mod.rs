//! Immutable persistent index implementations using the `imbl` library.
//! These indexes provide persistent data structures that allow for efficient
//! structural sharing and immutability.

mod btree;
pub use btree::BTree;

mod hashtable;
pub use hashtable::HashTable;

mod grouped;
pub use grouped::Grouped;

mod keys;
pub use keys::Keys;
