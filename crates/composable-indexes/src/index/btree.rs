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

    pub fn get_one(&self, key: &T) -> Option<&Key>
    where
        T: Ord + Eq,
    {
        let key = self.data.get(key).and_then(|v| v.iter().next());
        key
    }

    pub fn get_all(&self, key: &T) -> Vec<&Key>
    where
        T: Ord + Eq,
    {
        let keys = self.data.get(key);
        keys.map(|v| v.iter()).unwrap_or_default().collect()
    }

    pub fn range<R>(&self, range: R) -> Vec<&Key>
    where
        T: Ord + Eq,
        R: std::ops::RangeBounds<T>,
    {
        self.data
            .range(range)
            .flat_map(|(_, v)| v.iter())
            .collect()
    }

    pub fn min_one(&self) -> Option<&Key>
    where
        T: Ord + Eq,
    {
        self.data
            .iter()
            .next()
            .map(|(_, v)| v.iter().next().unwrap())
    }

    pub fn max_one(&self) -> Option<&Key>
    where
        T: Ord + Eq,
    {
        self.data
            .iter()
            .next_back()
            .map(|(_, v)| v.iter().next().unwrap())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::premap::premap;
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
            || premap(|i: &(Month, u8)| i.0, btree()),
            |q| {
                q.range(Month::Jan..=Month::Feb)
                    .iter()
                    .map(|i| i.1.clone())
                    .collect::<HashSet<u8>>()
            },
            |xs| {
                xs.iter()
                    .filter(|i| i.0 >= Month::Jan && i.0 <= Month::Feb)
                    .map(|i| i.1.clone())
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
