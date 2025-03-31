use crate::core::{Index, Insert, Key, QueryEnv, Remove, Update};
use std::{collections::HashMap, hash::Hash};

pub fn hashtable<T: Eq + std::hash::Hash>() -> HashTableIndex<T> {
    HashTableIndex {
        data: HashMap::new(),
    }
}

pub struct HashTableIndex<T> {
    data: HashMap<T, Key>,
}

impl<'t, In: Eq + Hash + Clone + 't> Index<'t, In> for HashTableIndex<In> {
    type Query<Out: 't> = HashTableQueries<'t, In, Out>;

    fn insert(&mut self, op: &Insert<In>) {
        self.data.insert(op.new.clone(), op.key);
    }

    fn update(&mut self, op: &Update<In>) {
        self.data.insert(op.new.clone(), op.key);
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.data.remove(&op.existing);
    }

    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
        HashTableQueries {
            data: &self.data,
            env,
        }
    }
}

pub struct HashTableQueries<'t, In, Out> {
    data: &'t HashMap<In, Key>,
    env: QueryEnv<'t, Out>,
}

impl<In: Eq + Hash, Out> HashTableQueries<'_, In, Out> {
    pub fn get(&self, key: &In) -> Option<&Out> {
        self.data.get(key).map(|k| self.env.data.get(k).unwrap())
    }
}
