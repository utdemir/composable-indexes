//! An index backed by [`std::collections::HashMap`]. Provides efficient
//! lookups by key with O(1) average time complexity.

use composable_indexes_core::{Index, Insert, Key, QueryEnv, Remove};
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
    type Query<'t, Out>
        = HashTableQueries<'t, In, Out>
    where
        Out: 't,
        Self: 't;

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

    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        HashTableQueries {
            data: &self.data,
            env,
        }
    }
}

pub struct HashTableQueries<'t, In, Out> {
    data: &'t HashMap<In, HashSet<Key>>,
    env: QueryEnv<'t, Out>,
}

impl<In: Eq + Hash, Out> HashTableQueries<'_, In, Out> {
    pub fn get_one(&self, key: &In) -> Option<&Out> {
        let key = self.data.get(key).and_then(|v| v.iter().next());
        key.map(|k| self.env.get(k))
    }

    pub fn get_all(&self, key: &In) -> Vec<&Out> {
        let keys = self.data.get(key);
        keys.map(|v| v.iter())
            .unwrap_or_default()
            .map(|k| self.env.get(k))
            .collect()
    }

    pub fn count_distinct(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes_testutils::prop_assert_reference;
    use proptest_derive::Arbitrary;
    use std::collections::HashSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary)]
    enum Month {
        Jan,
        Feb,
        Mar,
        Apr,
    }

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            || hashtable(),
            |q| {
                q.get_all(&Month::Mar)
                    .iter()
                    .map(|&month| month.clone())
                    .collect::<HashSet<_>>()
            },
            |xs| {
                xs.iter()
                    .filter(|&&month| month == Month::Mar)
                    .map(|&month| month.clone())
                    .collect::<HashSet<_>>()
            },
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            || hashtable::<u8>(),
            |q| q.count_distinct(),
            |xs| xs.iter().collect::<HashSet<_>>().len(),
            None,
        );
    }
}
