//! An alias to KeysIndex with an immutable backing store.

use crate::index;

pub type Keys = index::Keys<index::generic::DefaultImmutableKeySet>;
