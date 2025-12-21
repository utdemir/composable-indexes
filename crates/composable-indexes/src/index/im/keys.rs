//! An alias to KeysIndex with an immutable backing store.

use crate::index;

pub fn keys() -> KeysIndex {
    KeysIndex {
        keys: index::generic::DefaultImmutableKeySet::default(),
    }
}

pub type KeysIndex = index::KeysIndex<index::generic::DefaultImmutableKeySet>;
