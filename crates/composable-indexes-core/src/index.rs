use crate::{
    collection::{Insert, Remove, Update},
    Key,
};
use std::collections::HashMap;

/// Trait of indexes. You probably only need this if you're implementing a new index.
pub trait Index<In, Path: Clone> {
    type Query<'t, Out>
    where
        Self: 't,
        Out: 't;

    #[doc(hidden)]
    fn insert(&mut self, op: &Insert<In, Path>);

    #[doc(hidden)]
    fn remove(&mut self, op: &Remove<In, Path>);

    #[doc(hidden)]
    fn update(&mut self, op: &Update<In, Path>) {
        self.remove(&Remove {
            key: op.key.clone(),
            existing: op.existing,
        });
        self.insert(&Insert {
            key: op.key.clone(),
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
    pub fn get<Path>(&'t self, key: &Key<Path>) -> &'t T {
        self.data.get(&key.forget_path()).unwrap()
    }

    pub fn get_opt<Path>(&'t self, key: &Key<Path>) -> Option<&'t T> {
        self.data.get(&key.forget_path())
    }
}

impl<T> Clone for QueryEnv<'_, T> {
    fn clone(&self) -> Self {
        QueryEnv { data: self.data }
    }
}
