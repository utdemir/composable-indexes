//! An index backed by [`imbl::HashMap`]. Provides efficient
//! lookups by key with O(log n) time complexity using
//! persistent immutable data structures.

use alloc::vec::Vec;
use core::hash::Hash;

use imbl::HashMap;

use crate::{
    ShallowClone,
    core::{Index, Insert, Key, Remove, Seal},
    index::generic::{DefaultImmutableKeySet, KeySet},
};

#[derive(Clone)]
pub struct HashTable<T, KeySet = DefaultImmutableKeySet> {
    data: HashMap<T, KeySet>,
}

impl<T: Eq + Hash + Clone, KeySet_: KeySet + Default> Default for HashTable<T, KeySet_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash + Clone, KeySet_: KeySet + Default> HashTable<T, KeySet_> {
    pub fn new() -> Self {
        HashTable {
            data: HashMap::new(),
        }
    }
}

impl<T: Clone, KeySet_: Clone> ShallowClone for HashTable<T, KeySet_> {}

impl<In, KeySet_> Index<In> for HashTable<In, KeySet_>
where
    In: Eq + Hash + Clone,
    KeySet_: KeySet + Clone,
{
    fn insert(&mut self, _seal: Seal, op: &Insert<In>) {
        let mut set = self.data.get(op.new).cloned().unwrap_or_default();
        set.insert(op.key);
        self.data.insert(op.new.clone(), set);
    }

    fn remove(&mut self, _seal: Seal, op: &Remove<In>) {
        let existing = self.data.get_mut(op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(op.existing);
        }
    }
}

impl<In, KeySet_: KeySet> HashTable<In, KeySet_> {
    pub fn contains(&self, key: &In) -> bool
    where
        In: Eq + Hash,
    {
        self.data.contains_key(key)
    }

    pub fn count_distinct(&self) -> usize
    where
        In: Eq + Hash,
    {
        self.data.len()
    }

    pub fn get_one(&self, key: &In) -> Option<Key>
    where
        In: Eq + Hash,
    {
        self.data.get(key).and_then(|v| v.iter().next())
    }

    pub fn get_all(&self, key: &In) -> Vec<Key>
    where
        In: Eq + Hash,
    {
        self.data
            .get(key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> imbl::HashSet<Key> {
        self.data.values().flat_map(|keys| keys.iter()).collect()
    }
}

#[cfg(test)]
mod tests {
    use imbl::HashSet;

    use crate::testutils::prop_assert_reference;

    use super::*;

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            || HashTable::<u8>::new(),
            |db| db.query(|ix| ix.contains(&1)),
            |xs| xs.iter().find(|i| **i == 1).is_some(),
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            || HashTable::<u8>::new(),
            |db| db.query(|ix| ix.count_distinct()),
            |xs| xs.iter().cloned().collect::<HashSet<u8>>().len(),
            None,
        );
    }
}
