mod collection;
mod index;
mod query_result;
mod shallow_clone;
mod transaction;

pub mod store;

pub use collection::*;
pub use index::*;
pub use query_result::*;
pub use shallow_clone::*;
pub use transaction::*;

#[cfg(feature = "std")]
pub(crate) type DefaultHasher = std::hash::RandomState;

#[cfg(not(feature = "std"))]
pub(crate) type DefaultHasher = hashbrown::DefaultHashBuilder;

pub(crate) type DefaultStore<In> = hashbrown::HashMap<Key, In, DefaultHasher>;
