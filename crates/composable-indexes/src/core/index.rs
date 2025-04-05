use super::database::{Insert, Key, Remove, Update};
use std::collections::HashMap;

pub trait Index<'t, In> {
    type Query<Out>
    where
        Self: 't,
        Out: 't;

    fn insert(&mut self, op: &Insert<In>);
    fn remove(&mut self, op: &Remove<In>);

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

    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out>;
}

pub struct QueryEnv<'t, T> {
    pub data: &'t HashMap<Key, T>,
}

impl<'t, T> Clone for QueryEnv<'t, T> {
    fn clone(&self) -> Self {
        QueryEnv { data: self.data }
    }
}
