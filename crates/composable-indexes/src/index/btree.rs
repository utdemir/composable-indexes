//! An index backed by [`std::collections::BTreeMap`]. Provides efficient
//! queries for the minimum and maximum keys, and efficient lookups.

use composable_indexes_core::{Index, Insert, Key, QueryEnv, Remove};
use std::collections::{BTreeMap, HashSet};

pub fn btree<T: Ord + Eq>() -> BTreeIndex<T> {
    BTreeIndex {
        data: BTreeMap::new(),
    }
}

pub struct BTreeIndex<T> {
    data: BTreeMap<T, HashSet<Key>>,
}

impl<In: Ord + Clone> Index<In> for BTreeIndex<In> {
    type Query<'t, Out>
        = BTreeQueries<'t, In, Out>
    where
        Out: 't,
        Self: 't;

    fn insert(&mut self, op: &Insert<In>) {
        self.data.entry(op.new.clone()).or_default().insert(op.key);
    }

    fn remove(&mut self, op: &Remove<In>) {
        let existing = self.data.get_mut(&op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(&op.existing);
        }
    }

    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        BTreeQueries {
            data: &self.data,
            env,
        }
    }
}

pub struct BTreeQueries<'t, In, Out> {
    data: &'t BTreeMap<In, HashSet<Key>>,
    env: QueryEnv<'t, Out>,
}

impl<In: Ord + Eq, Out> BTreeQueries<'_, In, Out> {
    pub fn get_one(&self, key: &In) -> Option<&Out> {
        let key = self.data.get(key).map(|v| v.iter().next()).flatten();
        key.map(|k| self.env.get(k))
    }

    pub fn get_all(&self, key: &In) -> Vec<&Out> {
        let keys = self.data.get(key);
        keys.map(|v| v.iter())
            .unwrap_or_default()
            .map(|k| self.env.get(k))
            .collect()
    }

    pub fn range<R>(&self, range: R) -> Vec<&Out>
    where
        R: std::ops::RangeBounds<In>,
    {
        self.data
            .range(range)
            .flat_map(|(_, v)| v.iter())
            .map(|k| self.env.get(k))
            .collect()
    }

    pub fn max_one(&self) -> Option<&Out> {
        self.data
            .iter()
            .next_back()
            .map(|(_, v)| (v.iter().next().unwrap()))
            .map(|k| self.env.get(k))
    }

    pub fn min_one(&self) -> Option<&Out> {
        self.data
            .iter()
            .next()
            .map(|(_, v)| v.iter().next().unwrap())
            .map(|k| self.env.get(k))
    }

    pub fn count_distinct(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::premap::premap;
    use composable_indexes_props::prop_assert_reference;
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
    fn test_aggrs() {
        prop_assert_reference(
            || btree::<Month>(),
            |q| (q.max_one().cloned(), q.min_one().cloned()),
            |xs| (xs.iter().max().cloned(), xs.iter().min().cloned()),
            None,
        );
    }

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            || premap(|i: &(Month, u32)| i.1, btree()),
            |q| {
                q.get_all(&1)
                    .iter()
                    .map(|i| i.0.clone())
                    .collect::<HashSet<Month>>()
            },
            |xs| {
                xs.iter()
                    .filter(|i| i.1 == 1)
                    .map(|i| i.0.clone())
                    .collect::<HashSet<_>>()
            },
            None,
        );
    }

    #[test]
    fn test_range() {
        prop_assert_reference(
            || premap(|i: &(Month, u32)| i.1, btree()),
            |q| {
                q.range(0..=2)
                    .iter()
                    .map(|i| i.0.clone())
                    .collect::<HashSet<Month>>()
            },
            |xs| {
                xs.iter()
                    .filter(|i| i.1 <= 2)
                    .map(|i| i.0.clone())
                    .collect::<HashSet<_>>()
            },
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            || btree::<u8>(),
            |q| q.count_distinct(),
            |xs| xs.iter().collect::<HashSet<_>>().len(),
            None,
        );
    }
}
