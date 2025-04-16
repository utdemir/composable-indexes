//! Generic framework for building aggregation indexes.
//! Provides the base implementation for maintaining state and updating
//! aggregates as elements change in the collection.

use composable_indexes_core::{Index, Insert, QueryEnv, Remove};

pub struct AggregateIndex<In, Query, State> {
    current_state: State,
    query: fn(st: &State) -> Query,
    insert: fn(&mut State, &In),
    remove: fn(&mut State, &In),
}

impl<In, Query, State> AggregateIndex<In, Query, State> {
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
        }
    }
}

impl<In, Query, State> Index<In> for AggregateIndex<In, Query, State>
where
    State: 'static,
    Query: 'static,
    In: 'static,
{
    type Query<'t, Out>
        = Query
    where
        Out: 't;

    fn insert(&mut self, op: &Insert<In>) {
        (self.insert)(&mut self.current_state, op.new);
    }

    fn remove(&mut self, op: &Remove<In>) {
        (self.remove)(&mut self.current_state, op.existing);
    }

    fn query<'t, Out: 't>(&self, _env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        (self.query)(&self.current_state)
    }
}
