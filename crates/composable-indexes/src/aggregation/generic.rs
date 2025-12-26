//! Generic framework for building aggregation indexes.
//! Provides the base implementation for maintaining state and updating
//! aggregates as elements change in the collection.

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal},
};

#[derive(Clone)]
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
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<In>) {
        (self.insert)(&mut self.current_state, op.new);
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<In>) {
        (self.remove)(&mut self.current_state, op.existing);
    }
}

impl<In, Query: Clone, State> AggregateIndex<In, Query, State> {
    #[inline]
    pub fn get(&self) -> Query {
        (self.query)(&self.current_state)
    }
}

impl<In: Clone, Query: Clone, State: Clone> ShallowClone for AggregateIndex<In, Query, State> {}
