//! An index backed by [`imbl::HashMap`]. Provides efficient
//! lookups by key with O(log n) time complexity using
//! persistent immutable data structures.

use alloc::vec::Vec;
use core::hash::Hash;

use imbl::{HashMap, HashSet};

use crate::{
    ShallowClone,
    core::{Index, Insert, Key, Remove},
};

pub fn hashtable<T: Eq + Hash + Clone>() -> HashTableIndex<T> {
    HashTableIndex {
        data: HashMap::new(),
    }
}

#[derive(Clone)]
pub struct HashTableIndex<T> {
    data: HashMap<T, HashSet<Key>>,
}

impl<T: Clone> ShallowClone for HashTableIndex<T> {}

impl<In: Eq + Hash + Clone> Index<In> for HashTableIndex<In> {
    fn insert(&mut self, op: &Insert<In>) {
        let mut set = self.data.get(op.new).cloned().unwrap_or_else(HashSet::new);
        set.insert(op.key);
        self.data.insert(op.new.clone(), set);
    }

    fn remove(&mut self, op: &Remove<In>) {
        let existing = self.data.get_mut(op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(op.existing);
        }
    }
}

impl<In> HashTableIndex<In> {
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
        self.data.get(key).and_then(|v| v.iter().next()).cloned()
    }

    pub fn get_all(&self, key: &In) -> Vec<Key>
    where
        In: Eq + Hash,
    {
        self.data
            .get(key)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> HashSet<Key> {
        self.data
            .values()
            .flat_map(|keys| keys.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use imbl::HashSet;

    use crate::index::im::hashtable;
    use crate::testutils::prop_assert_reference;

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            || hashtable::<u8>(),
            |db| db.query(|ix| ix.contains(&1)),
            |xs| xs.iter().find(|i| **i == 1).is_some(),
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            || hashtable::<u8>(),
            |db| db.query(|ix| ix.count_distinct()),
            |xs| xs.iter().cloned().collect::<HashSet<u8>>().len(),
            None,
        );
    }
}
