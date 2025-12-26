//! Internal module containing core traits and types mostly useful for implementing your own indexes.

mod collection;
mod index;
mod query_result;
mod shallow_clone;

pub mod store;

pub use collection::*;
pub use index::*;
pub use query_result::*;
pub use shallow_clone::*;

pub(crate) type DefaultHasher = hashbrown::DefaultHashBuilder;
pub(crate) type DefaultStore<In> = hashbrown::HashMap<Key, In, DefaultHasher>;
