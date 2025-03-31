use seq_macro::seq;

pub mod btree;
pub use btree::btree;

pub mod premap;
pub use premap::premap;

pub mod grouped;
pub use grouped::grouped;

pub mod trivial;
pub use trivial::trivial;

pub mod filtered;
pub use filtered::filtered;

pub mod hashtable;
pub use hashtable::hashtable;

pub mod zip;
seq!(N in 2..=99 {
    pub use zip::zip~N;
});
