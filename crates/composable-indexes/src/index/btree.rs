//! An index backed by [`std::collections::BTreeMap`]. Provides efficient
//! queries for the minimum/maximum keys and range queries.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::core::{Index, Insert, Key, Remove};
use crate::index::generic::DefaultKeySet;
use crate::index::generic::KeySet;

pub fn btree<T: Ord + Clone>() -> BTreeIndex<T> {
    BTreeIndex {
        data: BTreeMap::new(),
    }
}

#[derive(Clone)]
pub struct BTreeIndex<T, KeySet = DefaultKeySet> {
    data: BTreeMap<T, KeySet>,
}

impl<T, KeySet_> Default for BTreeIndex<T, KeySet_>
where
    T: Ord + Clone,
    KeySet_: KeySet + Default,
{
    fn default() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}

impl<T, KeySet_> BTreeIndex<T, KeySet_>
where
    T: Ord + Clone,
    KeySet_: KeySet + Default,
{
    pub fn new() -> Self {
        Self::default()
    }
}

impl<In: Ord + Clone, KeySet_: KeySet> Index<In> for BTreeIndex<In, KeySet_> {
    #[inline]
    fn insert(&mut self, op: &Insert<In>) {
        self.data.entry(op.new.clone()).or_default().insert(op.key);
    }

    #[inline]
    fn remove(&mut self, op: &Remove<In>) {
        let existing = self.data.get_mut(op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(op.existing);
        }
    }
}

impl<T, KeySet_: KeySet> BTreeIndex<T, KeySet_> {
    #[inline]
    pub fn contains(&self, key: &T) -> bool
    where
        T: Ord + Eq,
    {
        self.data.contains_key(key)
    }

    #[inline]
    pub fn count_distinct(&self) -> usize
    where
        T: Ord + Eq,
    {
        self.data.len()
    }

    #[inline]
    pub fn get_one(&self, key: &T) -> Option<Key>
    where
        T: Ord + Eq,
    {
        self.data.get(key).and_then(|v| v.iter().next()).copied()
    }

    #[inline]
    pub fn get_all(&self, key: &T) -> Vec<Key>
    where
        T: Ord + Eq,
    {
        let keys = self.data.get(key);
        keys.map(|v| v.iter().copied().collect())
            .unwrap_or_default()
    }

    #[inline]
    pub fn range<R>(&self, range: R) -> Vec<Key>
    where
        T: Ord + Eq,
        R: core::ops::RangeBounds<T>,
    {
        self.data
            .range(range)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect()
    }

    #[inline]
    pub fn min_one(&self) -> Option<Key>
    where
        T: Ord + Eq,
    {
        self.data
            .iter()
            .next()
            .map(|(_, v)| *v.iter().next().unwrap())
    }

    #[inline]
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

impl BTreeIndex<String> {
    pub fn starts_with(&self, prefix: &str) -> Vec<Key> {
        let start = alloc::string::ToString::to_string(prefix);
        // Increment the last character to get the exclusive upper bound
        let mut end = start.clone();
        if let Some(last_char) = end.pop() {
            let next_char = (last_char as u8 + 1) as char;
            end.push(next_char);
        } else {
            end.push('\u{10FFFF}'); // Push the maximum valid Unicode character
        }

        self.data
            .range(start..end)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::index::premap::premap;
    use crate::testutils::{SortedVec, prop_assert_reference};
    use proptest_derive::Arbitrary;

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
            || BTreeIndex::<Month>::new(),
            |db| {
                let (mi, ma) = db.query(|ix| (ix.max_one(), ix.min_one()));
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
                db.query(|ix| ix.get_all(&1))
                    .into_iter()
                    .cloned()
                    .collect::<SortedVec<_>>()
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
                db.query(|ix| ix.range(Month::Jan..=Month::Feb))
                    .into_iter()
                    .cloned()
                    .collect::<SortedVec<_>>()
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
            |db| db.query(|ix| ix.count_distinct()),
            |xs| xs.iter().collect::<BTreeSet<_>>().len(),
            None,
        );
    }

    #[test]
    fn test_starts_with() {
        prop_assert_reference(
            || btree::<String>(),
            |db| {
                db.query(|ix| ix.starts_with("ab"))
                    .into_iter()
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            |xs| {
                xs.iter()
                    .filter(|s| s.starts_with("ab"))
                    .cloned()
                    .collect::<SortedVec<_>>()
            },
            None,
        );
    }
}
