use crate::core::{Index, Insert, Key, QueryEnv, Remove, Update};
use std::collections::BTreeMap;

pub fn btree<T: Ord + Eq>() -> BTreeIndex<T> {
    BTreeIndex {
        data: BTreeMap::new(),
    }
}

pub struct BTreeIndex<T> {
    data: BTreeMap<T, Key>,
}

impl<'t, In: Ord + Clone + 't, Out: 't> Index<'t, In, Out> for BTreeIndex<In> {
    type Queries = BTreeQueries<'t, In, Out>;

    fn insert(&mut self, op: &Insert<In>) {
        self.data.insert(op.new.clone(), op.key);
    }

    fn update(&mut self, op: &Update<In>) {
        self.data.insert(op.new.clone(), op.key);
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.data.remove(&op.existing);
    }

    fn query(&'t self, env: QueryEnv<'t, Out>) -> Self::Queries {
        BTreeQueries {
            data: &self.data,
            env,
        }
    }
}

pub struct BTreeQueries<'t, In, Out> {
    data: &'t BTreeMap<In, Key>,
    env: QueryEnv<'t, Out>,
}

impl<In: Ord + Eq, Out> BTreeQueries<'_, In, Out> {
    pub fn get(&self, key: &In) -> Option<&Out> {
        let key = self.data.get(key);
        key.map(|k| self.env.data.get(k).unwrap())
    }

    pub fn max(&self) -> Option<(&In, &Out)> {
        self.data
            .iter()
            .next_back()
            .map(|(i, v)| (i, self.env.data.get(v).unwrap()))
    }

    pub fn min(&self) -> Option<(&In, &Out)> {
        self.data
            .iter()
            .next()
            .map(|(i, v)| (i, self.env.data.get(v).unwrap()))
    }
}
