//! An index that maintains the keys of all received items.

use crate::{
    ShallowClone,
    core::{Index, Insert, Key, Remove},
    index::generic::{DefaultImmutableKeySet, KeySet},
};

pub fn keys() -> KeysIndex {
    KeysIndex {
        keys: DefaultImmutableKeySet::default(),
    }
}

#[derive(Clone)]
pub struct KeysIndex<KeySet = DefaultImmutableKeySet> {
    pub keys: KeySet,
}

impl<KeySet_: KeySet + Default> KeysIndex<KeySet_> {
    pub fn new() -> Self {
        KeysIndex {
            keys: KeySet_::default(),
        }
    }
}

impl<KeySet_: KeySet + Default> Default for KeysIndex<KeySet_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<KeySet_: Clone> ShallowClone for KeysIndex<KeySet_> {}

impl<In, KeySet_: KeySet> Index<In> for KeysIndex<KeySet_> {
    fn insert(&mut self, op: &Insert<In>) {
        self.keys.insert(op.key);
    }
    fn remove(&mut self, op: &Remove<In>) {
        self.keys.remove(&op.key);
    }
}
impl<KeySet_: KeySet> KeysIndex<KeySet_> {
    pub fn all(&self) -> impl Iterator<Item = Key> {
        self.keys.iter().copied()
    }

    pub fn contains(&self, key: &Key) -> bool {
        self.keys.contains(key)
    }

    pub fn count(&self) -> usize {
        self.keys.iter().count()
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
        let mut coll = Collection::<u8, _>::new(keys());

        let key1 = coll.insert(1);
        let key2 = coll.insert(2);

        assert!(coll.query(|ix| ix.contains(&key1)));
        assert!(coll.query(|ix| ix.contains(&key2)));
        assert_eq!(coll.query(|ix| ix.count()), 2);
    }

    #[test]
    fn test_all() {
        prop_assert_reference(
            || keys(),
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
