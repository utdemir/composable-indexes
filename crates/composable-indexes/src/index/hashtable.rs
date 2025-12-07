//! An index backed by [`std::collections::HashMap`]. Provides efficient
//! lookups by key with O(1) average time complexity.

use composable_indexes_core::{Index, Insert, Key, Remove};
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub fn hashtable<T: Eq + std::hash::Hash>() -> HashTableIndex<T> {
    HashTableIndex {
        data: HashMap::new(),
    }
}

pub struct HashTableIndex<T> {
    data: HashMap<T, HashSet<Key>>,
}

impl<In: Eq + Hash + Clone> Index<In> for HashTableIndex<In> {
    type Query = HashTableQueries<In>;

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

    pub fn get_one(&self, key: &In) -> Option<&Key>
    where
        In: Eq + Hash,
    {
        let key = self.data.get(key).and_then(|v| v.iter().next());
        key
    }

    pub fn get_all(&self, key: &In) -> Vec<&Key>
    where
        In: Eq + Hash,
    {
        let keys = self.data.get(key);
        keys.map(|v| v.iter()).unwrap_or_default().collect()
    }
}

pub struct HashTableQueries<In> {
    data: HashMap<In, HashSet<Key>>,
}

// impl<In: Eq + Hash, Out> HashTableQueries<'_, In, Out> {
//     pub fn get_one(&self, key: &In) -> Option<&Out> {
//         let key = self.data.get(key).and_then(|v| v.iter().next());
//         key.map(|k| self.env.get(k))
//     }

//     pub fn get_all(&self, key: &In) -> Vec<&Out> {
//         let keys = self.data.get(key);
//         keys.map(|v| v.iter())
//             .unwrap_or_default()
//             .map(|k| self.env.get(k))
//             .collect()
//     }

//     pub fn count_distinct(&self) -> usize {
//         self.data.len()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use composable_indexes_testutils::prop_assert_reference;
//     use proptest_derive::Arbitrary;
//     use std::collections::HashSet;

//     #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary)]
//     enum Month {
//         Jan,
//         Feb,
//         Mar,
//         Apr,
//     }

//     #[test]
//     fn test_lookup() {
//         prop_assert_reference(
//             || hashtable(),
//             |q| {
//                 q.get_all(&Month::Mar)
//                     .iter()
//                     .map(|&month| month.clone())
//                     .collect::<HashSet<_>>()
//             },
//             |xs| {
//                 xs.iter()
//                     .filter(|&&month| month == Month::Mar)
//                     .map(|&month| month.clone())
//                     .collect::<HashSet<_>>()
//             },
//             None,
//         );
//     }

//     #[test]
//     fn test_count_distinct() {
//         prop_assert_reference(
//             || hashtable::<u8>(),
//             |q| q.count_distinct(),
//             |xs| xs.iter().collect::<HashSet<_>>().len(),
//             None,
//         );
//     }
// }
