//! A basic index implementation that maintains no additional data structures.
//! Useful as a no-op index when indexing is not needed.

use composable_indexes_core::{Index, Insert, Key, QueryEnv, Remove, Update};

pub fn trivial() -> TrivialIndex {
    TrivialIndex
}

pub struct TrivialQueries<'t, Out> {
    env: QueryEnv<'t, Out>,
}

impl<Out> TrivialQueries<'_, Out> {
    pub fn get(&self, key: &Key) -> Option<&Out> {
        self.env.get_opt(key)
    }
}

pub struct TrivialIndex;

impl<In> Index<In> for TrivialIndex {
    type Query<'t, Out>
        = TrivialQueries<'t, Out>
    where
        Self: 't,
        Out: 't;
    fn insert(&mut self, _op: &Insert<In>) {}
    fn update(&mut self, _op: &Update<In>) {}
    fn remove(&mut self, _op: &Remove<In>) {}
    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        TrivialQueries { env }
    }
}
