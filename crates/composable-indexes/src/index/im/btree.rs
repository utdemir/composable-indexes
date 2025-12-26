//! An index backed by [`imbl::OrdMap`]. Provides efficient
//! queries for the minimum/maximum keys and range queries using
//! persistent immutable data structures.

use alloc::string::String;
use alloc::vec::Vec;

use imbl::OrdMap;

use crate::{
    ShallowClone,
    core::{Index, Insert, Key, Remove, Seal},
    index::generic::{DefaultImmutableKeySet, KeySet},
};

#[derive(Clone)]
pub struct BTreeIndex<T, KeySet = DefaultImmutableKeySet> {
    data: OrdMap<T, KeySet>,
}

impl<T: Ord + Clone, KeySet_: KeySet + Default> Default for BTreeIndex<T, KeySet_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone, KeySet_: KeySet + Default> BTreeIndex<T, KeySet_> {
    pub fn new() -> Self {
        BTreeIndex {
            data: OrdMap::new(),
        }
    }
}

impl<T: Clone, KeySet_: Clone> ShallowClone for BTreeIndex<T, KeySet_> {}

impl<In, KeySet_> Index<In> for BTreeIndex<In, KeySet_>
where
    In: Ord + Clone,
    KeySet_: KeySet + Clone,
{
    fn insert(&mut self, _seal: Seal, op: &Insert<In>) {
        self.data.entry(op.new.clone()).or_default().insert(op.key);
    }

    fn remove(&mut self, _seal: Seal, op: &Remove<In>) {
        let existing = self.data.get_mut(op.existing).unwrap();
        existing.remove(&op.key);
        if existing.is_empty() {
            self.data.remove(op.existing);
        }
    }
}

impl<T, KeySet_: KeySet> BTreeIndex<T, KeySet_> {
    pub fn contains(&self, key: &T) -> bool
    where
        T: Ord + Clone,
    {
        self.data.contains_key(key)
    }

    pub fn count_distinct(&self) -> usize
    where
        T: Ord + Clone,
    {
        self.data.len()
    }

    pub fn get_one(&self, key: &T) -> Option<Key>
    where
        T: Ord + Clone,
    {
        self.data.get(key).and_then(|v| v.iter().next())
    }

    pub fn get_all(&self, key: &T) -> Vec<Key>
    where
        T: Ord + Clone,
    {
        self.data
            .get(key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn range<R>(&self, range: R) -> Vec<Key>
    where
        T: Ord + Clone,
        R: core::ops::RangeBounds<T>,
    {
        self.data.range(range).flat_map(|(_, v)| v.iter()).collect()
    }

    pub fn min_one(&self) -> Option<Key>
    where
        T: Ord + Clone,
    {
        self.data.iter().next().and_then(|(_, v)| v.iter().next())
    }

    pub fn max_one(&self) -> Option<Key>
    where
        T: Ord + Clone,
    {
        self.data
            .iter()
            .next_back()
            .and_then(|(_, v)| v.iter().next())
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
    use super::*;
    use crate::index::premap::PremapOwned;
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
            || PremapOwned::new(|i: &(Month, u32)| i.1, BTreeIndex::<u32>::new()),
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
            || PremapOwned::new(|i: &(Month, u8)| i.0, BTreeIndex::<Month>::new()),
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
        use alloc::collections::BTreeSet;
        prop_assert_reference(
            BTreeIndex::<u8>::new,
            |db| db.query(|ix| ix.count_distinct()),
            |xs| xs.iter().collect::<BTreeSet<_>>().len(),
            None,
        );
    }

    #[test]
    fn test_starts_with() {
        prop_assert_reference(
            BTreeIndex::<String>::new,
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
