//! An index backed by [`std::collections::BTreeMap`]. Provides efficient
//! queries for the minimum/maximum keys and range queries.

use composable_indexes_core::{Index, Insert, Key, Remove};
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

impl<T> BTreeIndex<T> {
    pub fn contains(&self, key: &T) -> bool
    where
        T: Ord + Eq,
    {
        self.data.contains_key(key)
    }

    pub fn count_distinct(&self) -> usize
    where
        T: Ord + Eq,
    {
        self.data.len()
    }

    pub fn get_one(&self, key: &T) -> Option<Key>
    where
        T: Ord + Eq,
    {
        self.data.get(key).and_then(|v| v.iter().next()).copied()
    }

    pub fn get_all(&self, key: &T) -> Vec<Key>
    where
        T: Ord + Eq,
    {
        let keys = self.data.get(key);
        keys.map(|v| v.iter().copied()).unwrap_or_default().collect()
    }

    pub fn range<R>(&self, range: R) -> Vec<Key>
    where
        T: Ord + Eq,
        R: std::ops::RangeBounds<T>,
    {
        self.data
            .range(range)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect()
    }

    pub fn min_one(&self) -> Option<Key>
    where
        T: Ord + Eq,
    {
        self.data
            .iter()
            .next()
            .map(|(_, v)| *v.iter().next().unwrap())
    }

    pub fn max_one(&self) -> Option<Key>
    where
        T: Ord + Eq,
    {
        self.data
            .iter()
            .next_back()
            .map(|(_, v)| *v.iter().next().unwrap())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::premap::premap;
    use composable_indexes_core::Simple;
    use composable_indexes_testutils::{SortedVec, prop_assert_reference};
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
            |db| {
                let (mi, ma) = db.execute(|ix| (ix.max_one(), ix.min_one()));
                (mi.cloned(), ma.cloned())
            },
            |xs| {
                let max = xs.iter().max().cloned();
                let min = xs.iter().min().cloned();
                (max, min)
            },
            None,
        );
    }

    #[test]
    fn test_lookup() {
        prop_assert_reference(
            || premap(|i: &(Month, u32)| i.1, btree()),
            |db| {
                db.execute(|ix| {
                    ix.inner()
                        .get_all(&1)
                }).into_iter().cloned().collect::<SortedVec<_>>()
            },
            |xs| {
               xs.iter()
                    .filter(|i| i.1 == 1)
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            None,
        );
    }

    #[test]
    fn test_range() {
        prop_assert_reference(
            || premap(|i: &(Month, u8)| i.0, btree()),
            |db| {
                db.execute(|ix| {
                    ix.inner()
                        .range(Month::Jan..=Month::Feb)
                }).into_iter().cloned().collect::<SortedVec<_>>()
            },
            |xs| {
                xs.iter()
                    .filter(|i| i.0 >= Month::Jan && i.0 <= Month::Feb)
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            None,
        );
    }

    #[test]
    fn test_count_distinct() {
        prop_assert_reference(
            || btree::<u8>(),
            |db| db.execute(|ix| Simple(ix.count_distinct())),
            |xs| xs.iter().collect::<HashSet<_>>().len(),
            None,
        );
    }
}
