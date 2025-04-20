//! A combinator that groups entries by a key and maintains separate indexes for each group.
//! This enables functionality akin to the "group by" expression.

use composable_indexes_core::{Index, Insert, QueryEnv, Remove, Update};
use std::{collections::HashMap, hash::Hash};

pub fn grouped<InnerIndex, In, GroupKey, KeyFun>(
    group_key: KeyFun,
    mk_index: fn() -> InnerIndex,
) -> GroupedIndex<In, GroupKey, KeyFun, InnerIndex>
where
    GroupKey: Hash + Eq + Clone,
    KeyFun: Fn(&In) -> GroupKey,
{
    GroupedIndex::<In, GroupKey, KeyFun, InnerIndex> {
        group_key,
        mk_index,
        groups: std::collections::HashMap::new(),
        _marker: std::marker::PhantomData,
    }
}

pub struct GroupedIndex<T, GroupKey, KeyFun, InnerIndex> {
    group_key: KeyFun,
    mk_index: fn() -> InnerIndex,
    groups: std::collections::HashMap<GroupKey, InnerIndex>,
    _marker: std::marker::PhantomData<T>,
}

impl<In, GroupKey: Hash + Eq + Clone, KeyFun: Fn(&In) -> GroupKey, InnerIndex>
    GroupedIndex<In, GroupKey, KeyFun, InnerIndex>
{
    fn get_ix(&mut self, elem: &In) -> &mut InnerIndex {
        let key = (self.group_key)(elem);
        self.groups.entry(key).or_insert((self.mk_index)())
    }
}

impl<In, GroupKey: Hash + Eq + Clone, KeyFun: Fn(&In) -> GroupKey, InnerIndex: Index<In>> Index<In>
    for GroupedIndex<In, GroupKey, KeyFun, InnerIndex>
{
    type Query<'t, Out>
        = GroupedQueries<'t, In, GroupKey, KeyFun, InnerIndex, Out>
    where
        Self: 't,
        Out: 't;

    fn insert(&mut self, op: &Insert<In>) {
        self.get_ix(op.new).insert(op);
    }

    fn update(&mut self, op: &Update<In>) {
        let existing_key = (self.group_key)(&op.existing);
        let new_key = (self.group_key)(op.new);

        if existing_key == new_key {
            self.get_ix(op.new).update(op);
        } else {
            self.get_ix(&op.existing).remove(&Remove {
                key: op.key,
                existing: &op.existing,
            });
            self.get_ix(op.new).insert(&Insert {
                key: op.key,
                new: op.new,
            });
        }
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.get_ix(&op.existing).remove(op);
        // TODO: Remove empty groups
    }

    fn query<'t, Out: 't>(&'t self, _env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        GroupedQueries {
            empty_index: (self.mk_index)(),
            groups: &self.groups,
            env: _env,
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct GroupedQueries<'t, In, GroupKey, KeyFun, InnerIndex: 't, Out> {
    empty_index: InnerIndex,
    groups: &'t std::collections::HashMap<GroupKey, InnerIndex>,
    env: QueryEnv<'t, Out>,

    _marker: std::marker::PhantomData<(In, KeyFun)>,
}

impl<'t, In, GroupKey: Hash + Eq + Clone, KeyFun: Fn(&In) -> GroupKey, InnerIndex: Index<In>, Out>
    GroupedQueries<'t, In, GroupKey, KeyFun, InnerIndex, Out>
{
    pub fn get(&'t self, key: &GroupKey) -> InnerIndex::Query<'t, Out> {
        match self.groups.get(key) {
            Some(ix) => ix.query(self.env.clone()),
            None => self.empty_index.query(self.env.clone()),
        }
    }

    pub fn get_all(&'t self) -> HashMap<GroupKey, InnerIndex::Query<'t, Out>> {
        self.groups
            .iter()
            .map(|(key, ix)| (key.clone(), ix.query(self.env.clone())))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::sum;
    use crate::index::btree::btree;
    use crate::index::premap::premap;
    use composable_indexes_core::Collection;
    use composable_indexes_testutils::prop_assert_reference;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Payload {
        ty: String,
        value: u32,
    }

    fn sample_data() -> Vec<Payload> {
        vec![
            Payload {
                ty: "a".to_string(),
                value: 1,
            },
            Payload {
                ty: "b".to_string(),
                value: 2,
            },
            Payload {
                ty: "a".to_string(),
                value: 3,
            },
        ]
    }

    #[test]
    fn group_ix() {
        let mut db = Collection::<Payload, _>::new(grouped(
            |p: &Payload| p.ty.clone(),
            || premap(|p: &Payload| p.value, btree()),
        ));

        sample_data().into_iter().for_each(|p| {
            db.insert(p);
        });

        let q = db.query();

        assert_eq!(q.get(&"a".to_string()).max_one().map(|i| i.value), Some(3));
        assert_eq!(q.get(&"b".to_string()).max_one().map(|i| i.value), Some(2));
        assert_eq!(q.get(&"c".to_string()).max_one(), None);
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || grouped(|p: &u8| p % 4, || premap(|x| *x as u64, sum())),
            |q| {
                q.get_all()
                    .clone()
                    .iter()
                    .filter(|(_, v)| **v > 0)
                    .map(|(k, v)| (*k, *v))
                    .collect()
            },
            |xs| {
                let mut groups = std::collections::HashMap::new();
                for &x in xs {
                    let key = x % 4;
                    *groups.entry(key).or_insert(0) += x as u64;
                }
                groups
                    .into_iter()
                    .filter(|(_, v)| *v > 0)
                    .collect::<HashMap<_, _>>()
            },
            None,
        );
    }
}
