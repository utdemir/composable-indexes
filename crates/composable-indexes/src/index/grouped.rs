//! A combinator that groups entries by a key and maintains separate indexes for each group.
//! This enables functionality similar to the "group by" expression.

use core::hash::{BuildHasher, Hash};

use crate::{
    aggregation,
    core::{DefaultHasher, Index, Insert, Remove, Update},
    index::{ZipIndex2, zip::zip2},
};
use hashbrown::HashMap;

pub fn grouped<InnerIndex, In, GroupKey>(
    group_key: fn(&In) -> GroupKey,
    mk_index: fn() -> InnerIndex,
) -> GroupedIndex<In, GroupKey, InnerIndex> {
    GroupedIndex::<In, GroupKey, InnerIndex> {
        group_key,
        mk_index,
        empty: mk_index(),
        groups: HashMap::with_hasher(DefaultHasher::default()),
        _marker: core::marker::PhantomData,
    }
}

pub fn grouped_with_hasher<InnerIndex, In, GroupKey, S>(
    group_key: fn(&In) -> GroupKey,
    mk_index: fn() -> InnerIndex,
    hasher: S,
) -> GroupedIndex<In, GroupKey, InnerIndex, S>
where
    S: core::hash::BuildHasher,
{
    GroupedIndex::<In, GroupKey, InnerIndex, S> {
        group_key,
        mk_index,
        empty: mk_index(),
        groups: HashMap::with_hasher(hasher),
        _marker: core::marker::PhantomData,
    }
}

#[derive(Clone)]
pub struct GroupedIndex<T, GroupKey, InnerIndex, S = DefaultHasher> {
    group_key: fn(&T) -> GroupKey,
    mk_index: fn() -> InnerIndex,
    groups: HashMap<GroupKey, ZipIndex2<T, InnerIndex, aggregation::CountIndex>, S>,
    empty: InnerIndex,
    _marker: core::marker::PhantomData<fn() -> T>,
}

impl<In, GroupKey, InnerIndex, S> GroupedIndex<In, GroupKey, InnerIndex, S>
where
    GroupKey: Eq + Hash,
    S: BuildHasher,
{
    fn get_ix(&mut self, elem: &In) -> &mut ZipIndex2<In, InnerIndex, aggregation::CountIndex> {
        let key = (self.group_key)(elem);
        self.groups.entry(key).or_insert_with(|| {
            let ix = (self.mk_index)();
            zip2(ix, aggregation::count())
        })
    }
}

impl<In, GroupKey, InnerIndex, S> Index<In> for GroupedIndex<In, GroupKey, InnerIndex, S>
where
    GroupKey: Eq + Hash,
    InnerIndex: Index<In>,
    S: BuildHasher,
{
    #[inline]
    fn insert(&mut self, op: &Insert<In>) {
        self.get_ix(op.new).insert(op);
    }

    #[inline]
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

    #[inline]
    fn remove(&mut self, op: &Remove<In>) {
        let key = (self.group_key)(op.existing);
        let ix = self.groups.get_mut(&key).unwrap();
        ix.remove(op);
        if ix._2().get() == 0 {
            self.groups.remove(&key);
        }
    }
}

impl<In, GroupKey: Eq + Hash, InnerIndex, S: BuildHasher>
    GroupedIndex<In, GroupKey, InnerIndex, S>
{
    #[inline]
    pub fn get(&self, key: &GroupKey) -> &InnerIndex {
        self.groups.get(key).map(|i| i._1()).unwrap_or(&self.empty)
    }

    pub fn groups(&self) -> impl Iterator<Item = (&GroupKey, &InnerIndex)> {
        self.groups.iter().map(|(k, v)| (k, v._1()))
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
