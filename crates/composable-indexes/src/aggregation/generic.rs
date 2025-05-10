//! Generic framework for building aggregation indexes.
//! Provides the base implementation for maintaining state and updating
//! aggregates as elements change in the collection.

use composable_indexes_core::{Index, Insert, QueryEnv, Remove};

pub struct AggregateIndex<In, Query, State, Path> {
    current_state: State,
    query: fn(st: &State) -> Query,
    insert: fn(&mut State, &In),
    remove: fn(&mut State, &In),

    _marker: std::marker::PhantomData<(In, Path)>,
}

impl<In, Query, State, Path> AggregateIndex<In, Query, State, Path> {
    pub fn new(
        initial_state: State,
        query: fn(&State) -> Query,
        insert: fn(&mut State, &In),
        remove: fn(&mut State, &In),
    ) -> Self {
        Self {
            current_state: initial_state,
            query,
            insert,
            remove,

            _marker: std::marker::PhantomData,
        }
    }
}

impl<In, Query, State, Path: Clone> Index<In, Path> for AggregateIndex<In, Query, State, Path>
where
    State: 'static,
    Query: 'static,
    In: 'static,
    Path: 'static,
{
    type Query<'t, Out>
        = Query
    where
        Out: 't,
        Path: 't;

    fn insert(&mut self, op: &Insert<In, Path>) {
        (self.insert)(&mut self.current_state, op.new);
    }

    fn remove(&mut self, op: &Remove<In, Path>) {
        (self.remove)(&mut self.current_state, op.existing);
    }

    fn query<'t, Out: 't>(&self, _env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        (self.query)(&self.current_state)
    }
}
