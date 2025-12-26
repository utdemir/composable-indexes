//! An index backed by [`std::collections::HashMap`]. Provides efficient
//! lookups by key with O(1) average time complexity.

use alloc::vec::Vec;
use core::hash::Hash;
use hashbrown::HashMap;

use crate::core::{DefaultHasher, Index, Insert, Key, Remove, Seal};
use crate::index::generic::{DefaultKeySet, KeySet};

pub fn hashtable<T: Eq + core::hash::Hash>() -> HashTableIndex<T> {
    HashTableIndex {
        data: HashMap::with_hasher(DefaultHasher::default()),
    }
}

pub fn hashtable_with_hasher<T: Eq + core::hash::Hash, S: core::hash::BuildHasher>(
    hasher: S,
) -> HashTableIndex<T, S> {
    HashTableIndex {
        data: HashMap::with_hasher(hasher),
    }
}

#[derive(Clone)]
pub struct HashTableIndex<T, S = DefaultHasher, KeySet = DefaultKeySet> {
    data: HashMap<T, KeySet, S>,
}

impl<T, S, KeySet_> Default for HashTableIndex<T, S, KeySet_>
where
    T: Eq + Hash,
    S: core::hash::BuildHasher + Default,
    KeySet_: KeySet + Default,
{
    fn default() -> Self {
        HashTableIndex {
            data: HashMap::with_hasher(S::default()),
        }
    }
}

impl<T, S, KeySet_> HashTableIndex<T, S, KeySet_>
where
    T: Eq + Hash,
    S: core::hash::BuildHasher + Default,
    KeySet_: KeySet + Default,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<In, S, KeySet_> Index<In> for HashTableIndex<In, S, KeySet_>
where
    In: Eq + Hash + Clone,
    S: core::hash::BuildHasher,
    KeySet_: KeySet,
{
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<In>) {
        self.data.entry(op.new.clone()).or_default().insert(op.key);
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<In>) {
        let existing = self.data.get_mut(op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(op.existing);
        }
    }
}

impl<In, S, KeySet_> HashTableIndex<In, S, KeySet_>
where
    S: core::hash::BuildHasher,
    KeySet_: KeySet,
{
    #[inline]
    pub fn contains(&self, key: &In) -> bool
    where
        In: Eq + Hash,
    {
        self.data.contains_key(key)
    }

    #[inline]
    pub fn count_distinct(&self) -> usize
    where
        In: Eq + Hash,
    {
        self.data.len()
    }

    #[inline]
    pub fn get_one(&self, key: &In) -> Option<Key>
    where
        In: Eq + Hash,
    {
        self.data.get(key).and_then(|v| v.iter().next())
    }

    #[inline]
    pub fn get_all(&self, key: &In) -> Vec<Key>
    where
        In: Eq + Hash,
    {
        self.data
            .get(key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> hashbrown::HashSet<Key> {
        self.data.values().flat_map(|keys| keys.iter()).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{index::hashtable, testutils::prop_assert_reference};

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            hashtable::<u8>,
            |db| db.query(|ix| ix.contains(&1)),
            |xs| xs.contains(&1),
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            hashtable::<u8>,
            |db| db.query(|ix| ix.count_distinct()),
            |xs| xs.iter().cloned().collect::<HashSet<u8>>().len(),
            None,
        );
    }
}
