//! A combinator that groups entries by a key and maintains separate indexes for each group.
//! This enables functionality similar to the "group by" expression.

use crate::core::{Index, Insert, Remove, Update};
use alloc::collections::BTreeMap;

pub fn grouped<InnerIndex, In, GroupKey>(
    group_key: fn(&In) -> GroupKey,
    mk_index: fn() -> InnerIndex,
) -> GroupedIndex<In, GroupKey, InnerIndex> {
    GroupedIndex::<In, GroupKey, InnerIndex> {
        group_key,
        mk_index,
        empty: mk_index(),
        groups: BTreeMap::new(),
        _marker: core::marker::PhantomData,
    }
}

#[derive(Clone)]
pub struct GroupedIndex<T, GroupKey, InnerIndex> {
    group_key: fn(&T) -> GroupKey,
    mk_index: fn() -> InnerIndex,
    // TODO: Faster if we use a hashmap
    groups: BTreeMap<GroupKey, InnerIndex>,
    empty: InnerIndex,
    _marker: core::marker::PhantomData<fn() -> T>,
}

impl<In, GroupKey: Ord + Clone, InnerIndex> GroupedIndex<In, GroupKey, InnerIndex> {
    fn get_ix(&mut self, elem: &In) -> &mut InnerIndex {
        let key = (self.group_key)(elem);
        self.groups.entry(key).or_insert((self.mk_index)())
    }
}

impl<In, GroupKey: Ord + Eq + Clone, InnerIndex: Index<In>> Index<In>
    for GroupedIndex<In, GroupKey, InnerIndex>
{
    fn insert(&mut self, op: &Insert<In>) {
        self.get_ix(op.new).insert(op);
    }

    fn update(&mut self, op: &Update<In>) {
        let existing_key = (self.group_key)(op.existing);
        let new_key = (self.group_key)(op.new);

        if existing_key == new_key {
            self.get_ix(op.new).update(op);
        } else {
            self.get_ix(op.existing).remove(&Remove {
                key: op.key,
                existing: op.existing,
            });
            self.get_ix(op.new).insert(&Insert {
                key: op.key,
                new: op.new,
            });
        }
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.get_ix(op.existing).remove(op);
        // TODO: Remove empty groups
    }
}

impl<In, GroupKey: Ord + Clone, InnerIndex> GroupedIndex<In, GroupKey, InnerIndex> {
    pub fn get(&self, key: &GroupKey) -> &InnerIndex {
        self.groups.get(key).unwrap_or(&self.empty)
    }

    pub fn get_if_nonempty(&self, key: &GroupKey) -> Option<&InnerIndex> {
        self.groups.get(key)
    }

    pub fn groups(&self) -> &BTreeMap<GroupKey, InnerIndex> {
        &self.groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation::sum;
    use crate::core::Collection;
    use crate::index::btree::btree;
    use crate::index::premap::premap;
    use crate::testutils::{SortedVec, prop_assert_reference};

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

        let a_max = db.query(|ix| ix.get(&"a".to_string()).inner().max_one());
        assert_eq!(a_max.as_ref().map(|p| p.value), Some(3));

        let b_max = db.query(|ix| ix.get(&"b".to_string()).inner().max_one());
        assert_eq!(b_max.as_ref().map(|p| p.value), Some(2));

        let c_max = db.query(|ix| ix.get(&"c".to_string()).inner().max_one());
        assert_eq!(c_max, None);
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || grouped(|p: &u8| p % 4, || premap(|x| *x as u64, sum())),
            |db| {
                db.query(|ix| {
                    ix.groups()
                        .iter()
                        .map(|(k, v)| (*k, v.inner().get()))
                        .filter(|(_, v)| *v > 0)
                        .collect::<Vec<_>>()
                })
                .into()
            },
            |xs| {
                let mut groups = std::collections::HashMap::new();
                for x in xs {
                    let key = x % 4;
                    *groups.entry(key).or_insert(0) += x as u64;
                }
                groups
                    .into_iter()
                    .filter(|(_, v)| *v > 0)
                    .collect::<SortedVec<_>>()
            },
            None,
        );
    }
}
