//! An index that maintains the keys of all received items.

use crate::ShallowClone;
use crate::core::{Index, Insert, Key, Remove, Seal};
use crate::index::generic::{DefaultKeySet, KeySet};

#[derive(Clone)]
pub struct Keys<KeySet = DefaultKeySet> {
    pub keys: KeySet,
}

impl<KeySet_: KeySet + Default> Default for Keys<KeySet_> {
    fn default() -> Self {
        Keys {
            keys: KeySet_::default(),
        }
    }
}

impl<KeySet: ShallowClone> ShallowClone for Keys<KeySet> {}

impl Keys<DefaultKeySet> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<KeySet_: KeySet> Keys<KeySet_> {
    pub fn with_keyset() -> Self {
        Self::default()
    }
}

impl<In, KeySet_: KeySet> Index<In> for Keys<KeySet_> {
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<In>) {
        self.keys.insert(op.key);
    }
    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<In>) {
        self.keys.remove(&op.key);
    }
}
impl<KeySet_: KeySet> Keys<KeySet_> {
    pub fn all(&self) -> impl Iterator<Item = Key> {
        self.keys.iter()
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.keys.contains(key)
    }

    pub fn count(&self) -> usize {
        self.keys.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::Collection,
        testutils::{SortedVec, prop_assert_reference},
    };

    #[test]
    fn test_basic() {
        let mut coll = Collection::<u8, _>::new(Keys::new());

        let key1 = coll.insert(1);
        let key2 = coll.insert(2);

        assert!(coll.query(|ix| ix.contains(&key1)));
        assert!(coll.query(|ix| ix.contains(&key2)));
        assert_eq!(coll.query(|ix| ix.count()), 2);
    }

    #[test]
    fn test_all() {
        prop_assert_reference(
            Keys::new,
            |db| {
                db.query(|ix| ix.all().collect::<Vec<_>>())
                    .into_iter()
                    .copied()
                    .collect::<SortedVec<_>>()
            },
            |xs: Vec<u8>| xs.into_iter().collect::<SortedVec<_>>(),
            None,
        );
    }
}
