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

impl<'t, In: Ord + Clone + 't> Index<'t, In> for BTreeIndex<In> {
    type Query<Out: 't> = BTreeQueries<'t, In, Out>;

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

    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
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
        key.map(|k| self.env.data.get(k).unwrap())
    }

    pub fn get_all(&self, key: &In) -> Vec<&Out> {
        let keys = self.data.get(key);
        keys.map(|v| v.iter())
            .unwrap_or_default()
            .map(|k| self.env.data.get(k).unwrap())
            .collect()
    }

    pub fn max_one(&self) -> Option<(&In, &Out)> {
        self.data
            .iter()
            .next_back()
            .map(|(i, v)| (i, v.iter().next().unwrap()))
            .map(|(i, k)| (i, self.env.data.get(k).unwrap()))
    }

    pub fn min_one(&self) -> Option<(&In, &Out)> {
        self.data
            .iter()
            .next()
            .map(|(i, v)| (i, v.iter().next().unwrap()))
            .map(|(i, k)| (i, self.env.data.get(k).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexes::premap::premap;
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
            |q| {
                (
                    q.max_one().map(|(_k, v)| v).cloned(),
                    q.min_one().map(|(_k, v)| v).cloned(),
                )
            },
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
}
