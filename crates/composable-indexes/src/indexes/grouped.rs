use std::hash::Hash;

use crate::{Index, Insert, Remove};

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

impl<
    't,
    In: 't,
    GroupKey: Hash + Eq + Clone + 't,
    KeyFun: Fn(&In) -> GroupKey + 't,
    InnerIndex: Index<'t, In> + 't,
> Index<'t, In> for GroupedIndex<In, GroupKey, KeyFun, InnerIndex>
{
    type Query<Out: 't> = GroupedQueries<'t, In, GroupKey, KeyFun, InnerIndex, Out>;

    fn insert(&mut self, op: &crate::Insert<In>) {
        self.get_ix(op.new).insert(op);
    }

    fn update(&mut self, op: &crate::Update<In>) {
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

    fn remove(&mut self, op: &crate::Remove<In>) {
        self.get_ix(&op.existing).remove(op);
        // TODO: Remove empty groups
    }

    fn query<Out>(&'t self, _env: crate::QueryEnv<'t, Out>) -> Self::Query<Out> {
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
    env: crate::QueryEnv<'t, Out>,

    _marker: std::marker::PhantomData<(In, KeyFun)>,
}

impl<
    't,
    In,
    GroupKey: Hash + Eq + Clone,
    KeyFun: Fn(&In) -> GroupKey,
    InnerIndex: Index<'t, In>,
    Out,
> GroupedQueries<'t, In, GroupKey, KeyFun, InnerIndex, Out>
{
    pub fn get(&'t self, key: &GroupKey) -> InnerIndex::Query<Out> {
        match self.groups.get(key) {
            Some(ix) => ix.query(self.env.clone()),
            None => self.empty_index.query(self.env.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;
    use crate::indexes::btree::btree;
    use crate::indexes::premap::premap;

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

        assert_eq!(q.get(&"a".to_string()).max_one().map(|i| i.0), Some(&3));
        assert_eq!(q.get(&"b".to_string()).max_one().map(|i| i.0), Some(&2));
        assert_eq!(q.get(&"c".to_string()).max_one(), None);
    }
}
