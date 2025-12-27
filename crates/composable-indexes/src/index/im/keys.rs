use crate::index;

/// An alias to KeysIndex with an immutable backing store.
pub type Keys = index::Keys<index::generic::DefaultImmutableKeySet>;

impl Keys {
    pub fn new_immutable() -> Self {
        Self::default()
    }
}
