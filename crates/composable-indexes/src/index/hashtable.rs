//! An index backed by [`std::collections::HashMap`]. Provides efficient
//! lookups by key with O(1) average time complexity.

use crate::compat::{HashMap, HashSet};
use alloc::vec::Vec;
use core::hash::Hash;

use crate::core::{Index, Insert, Key, Remove};

pub fn hashtable<T: Eq + core::hash::Hash>() -> HashTableIndex<T> {
    HashTableIndex {
        data: HashMap::new(),
    }
}

pub fn hashtable_with_hasher<T: Eq + core::hash::Hash, S: core::hash::BuildHasher>(
    hasher: S,
) -> HashTableIndex<T, S> {
    HashTableIndex {
        data: HashMap::with_hasher(hasher),
    }
}

pub struct HashTableIndex<T, S = crate::compat::DefaultHashBuilder> {
    data: HashMap<T, HashSet<Key>, S>,
}

impl<In: Eq + Hash + Clone> Index<In> for HashTableIndex<In> {
    fn insert(&mut self, op: &Insert<In>) {
        self.data.entry(op.new.clone()).or_default().insert(op.key);
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
            .map(|v| v.iter().cloned())
            .unwrap_or_default()
            .collect()
    }

    pub fn all(&self) -> HashSet<Key> {
        self.data
            .values()
            .flat_map(|keys| keys.iter().cloned())
            .collect()
    }
}
