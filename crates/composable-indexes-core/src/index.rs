use crate::collection::{Insert, Key, Remove, Update};
use std::collections::HashMap;

/// Trait of indexes. You probably only need this if you're implementing a new index.
pub trait Index<In> {
    type Query<'t, Out>
    where
        Self: 't,
        Out: 't;

    #[doc(hidden)]
    fn insert(&mut self, op: &Insert<In>);

    #[doc(hidden)]
    fn remove(&mut self, op: &Remove<In>);

    #[doc(hidden)]
    fn update(&mut self, op: &Update<In>) {
        self.remove(&Remove {
            key: op.key,
            existing: op.existing,
        });
        self.insert(&Insert {
            key: op.key,
            new: op.new,
        });
    }

    #[doc(hidden)]
    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out>;
}

pub struct QueryEnv<'t, T> {
    pub(crate) data: &'t HashMap<Key, T>,
}

impl<'t, T> QueryEnv<'t, T> {
    pub fn get(&'t self, key: &Key) -> &'t T {
        self.data.get(key).unwrap()
    }

    pub fn get_opt(&'t self, key: &Key) -> Option<&'t T> {
        self.data.get(key)
    }
}

impl<T> Clone for QueryEnv<'_, T> {
    fn clone(&self) -> Self {
        QueryEnv { data: self.data }
    }
}
