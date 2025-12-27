//! A combinator that groups entries by a key and maintains separate indexes for each group.
//! This enables functionality similar to the "group by" expression.

use core::hash::{BuildHasher, Hash};

use crate::{
    aggregation,
    core::{DefaultHasher, Index, Insert, Remove, Seal, Update},
    index::Zip2,
};
use hashbrown::HashMap;

/// Generic grouped index that takes a function as a type parameter
#[derive(Clone)]
pub struct GenericGrouped<T, GroupKey, InnerIndex, F, S = DefaultHasher> {
    group_key: F,
    mk_index: fn() -> InnerIndex,
    groups: HashMap<GroupKey, Zip2<T, InnerIndex, aggregation::Count>, S>,
    empty: InnerIndex,
    _marker: core::marker::PhantomData<fn() -> T>,
}

/// Type alias for grouped index with references (function returns &GroupKey)
pub type Grouped<T, GroupKey, InnerIndex, S = DefaultHasher> =
    GenericGrouped<T, GroupKey, InnerIndex, fn(&T) -> &GroupKey, S>;

/// Type alias for grouped index with owned values (function returns GroupKey)
pub type GroupedOwned<T, GroupKey, InnerIndex, S = DefaultHasher> =
    GenericGrouped<T, GroupKey, InnerIndex, fn(&T) -> GroupKey, S>;

impl<In, GroupKey, InnerIndex> Grouped<In, GroupKey, InnerIndex> {
    pub fn new(group_key: fn(&In) -> &GroupKey, mk_index: fn() -> InnerIndex) -> Self {
        GenericGrouped {
            group_key,
            mk_index,
            empty: mk_index(),
            groups: HashMap::with_hasher(DefaultHasher::default()),
            _marker: core::marker::PhantomData,
        }
    }

    pub fn with_hasher<S: core::hash::BuildHasher>(
        group_key: fn(&In) -> &GroupKey,
        mk_index: fn() -> InnerIndex,
        hasher: S,
    ) -> GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> &GroupKey, S> {
        GenericGrouped {
            group_key,
            mk_index,
            empty: mk_index(),
            groups: HashMap::with_hasher(hasher),
            _marker: core::marker::PhantomData,
        }
    }
}

impl<In, GroupKey, InnerIndex> GroupedOwned<In, GroupKey, InnerIndex> {
    pub fn new(group_key: fn(&In) -> GroupKey, mk_index: fn() -> InnerIndex) -> Self {
        GenericGrouped {
            group_key,
            mk_index,
            empty: mk_index(),
            groups: HashMap::with_hasher(DefaultHasher::default()),
            _marker: core::marker::PhantomData,
        }
    }

    pub fn with_hasher<S: core::hash::BuildHasher>(
        group_key: fn(&In) -> GroupKey,
        mk_index: fn() -> InnerIndex,
        hasher: S,
    ) -> GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> GroupKey, S> {
        GenericGrouped {
            group_key,
            mk_index,
            empty: mk_index(),
            groups: HashMap::with_hasher(hasher),
            _marker: core::marker::PhantomData,
        }
    }
}

// Implementation for reference-based grouped index
impl<In, GroupKey, InnerIndex, S> GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> &GroupKey, S>
where
    GroupKey: Eq + Hash + Clone,
    S: BuildHasher,
{
    fn get_ix(&mut self, elem: &In) -> &mut Zip2<In, InnerIndex, aggregation::Count> {
        let key = (self.group_key)(elem);
        self.groups
            .raw_entry_mut()
            .from_key(key)
            .or_insert_with(|| {
                let ix = (self.mk_index)();
                let count_ix = aggregation::Count::new();
                (key.clone(), Zip2::new(ix, count_ix))
            })
            .1
    }
}

impl<In, GroupKey, InnerIndex, S> Index<In>
    for GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> &GroupKey, S>
where
    GroupKey: Eq + Hash + Clone,
    InnerIndex: Index<In>,
    S: BuildHasher,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<In>) {
        self.get_ix(op.new).insert(seal, op);
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<In>) {
        let existing_key = (self.group_key)(op.existing);
        let new_key = (self.group_key)(op.new);

        if existing_key == new_key {
            self.get_ix(op.new).update(seal, op);
        } else {
            self.get_ix(op.existing).remove(
                seal,
                &Remove {
                    key: op.key,
                    existing: op.existing,
                },
            );
            self.get_ix(op.new).insert(
                seal,
                &Insert {
                    key: op.key,
                    new: op.new,
                },
            );
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<In>) {
        let key = (self.group_key)(op.existing);
        let ix = self.groups.get_mut(key).unwrap();
        ix.remove(seal, op);
        if ix._2().get() == 0 {
            self.groups.remove(key);
        }
    }
}

// Implementation for owned-based grouped index
impl<In, GroupKey, InnerIndex, S> GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> GroupKey, S>
where
    GroupKey: Eq + Hash + Clone,
    S: BuildHasher,
{
    fn get_ix(&mut self, elem: &In) -> &mut Zip2<In, InnerIndex, aggregation::Count> {
        let key = (self.group_key)(elem);
        self.groups.entry(key).or_insert_with(|| {
            let ix = (self.mk_index)();
            let count_ix = aggregation::Count::new();
            Zip2::new(ix, count_ix)
        })
    }
}

impl<In, GroupKey, InnerIndex, S> Index<In>
    for GenericGrouped<In, GroupKey, InnerIndex, fn(&In) -> GroupKey, S>
where
    GroupKey: Eq + Hash + Clone,
    InnerIndex: Index<In>,
    S: BuildHasher,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<In>) {
        self.get_ix(op.new).insert(seal, op);
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<In>) {
        let existing_key = (self.group_key)(op.existing);
        let new_key = (self.group_key)(op.new);

        if existing_key == new_key {
            self.get_ix(op.new).update(seal, op);
        } else {
            let existing_key = (self.group_key)(op.existing);
            self.get_ix(op.existing).remove(
                seal,
                &Remove {
                    key: op.key,
                    existing: op.existing,
                },
            );
            let ix = self.groups.get_mut(&existing_key).unwrap();
            if ix._2().get() == 0 {
                self.groups.remove(&existing_key);
            }

            self.get_ix(op.new).insert(
                seal,
                &Insert {
                    key: op.key,
                    new: op.new,
                },
            );
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<In>) {
        let key = (self.group_key)(op.existing);
        let ix = self.groups.get_mut(&key).unwrap();
        ix.remove(seal, op);
        if ix._2().get() == 0 {
            self.groups.remove(&key);
        }
    }
}

impl<In, GroupKey: Eq + Hash, InnerIndex, F, S: BuildHasher>
    GenericGrouped<In, GroupKey, InnerIndex, F, S>
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
    use crate::core::Collection;
    use crate::index;
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
        let mut db = Collection::<Payload, _>::new(GroupedOwned::new(
            |p: &Payload| p.ty.clone(),
            || index::PremapOwned::new(|p: &Payload| p.value, index::BTree::<u32>::new()),
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
            || {
                GroupedOwned::new(
                    |p: &u8| p % 4,
                    || index::PremapOwned::new(|x| *x as u64, aggregation::Sum::new()),
                )
            },
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
