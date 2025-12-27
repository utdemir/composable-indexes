//! Generic framework for building aggregation indexes.
//! Provides the base implementation for maintaining state and updating
//! aggregates as elements change in the collection.

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal, Update},
};

#[derive(Clone)]
pub struct GenericAggregate<In, Query, State> {
    current_state: State,
    query: fn(st: &State) -> Query,
    insert: fn(&mut State, &In),
    remove: fn(&mut State, &In),
}

impl<In, Query, State> GenericAggregate<In, Query, State> {
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

impl<In, Query, State> Index<In> for GenericAggregate<In, Query, State>
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

impl<In, Query: Clone, State> GenericAggregate<In, Query, State> {
    #[inline]
    pub fn get(&self) -> Query {
        (self.query)(&self.current_state)
    }
}

impl<In: Clone, Query: Clone, State: Clone> ShallowClone for GenericAggregate<In, Query, State> {}

pub struct MonoidalAggregate<T, S, O> {
    state: S,
    input: fn(&T) -> S,
    combine: fn(&S, &S) -> S,
    invert: fn(&S) -> S,
    output: fn(&S) -> O,
}

impl<T, S, O> MonoidalAggregate<T, S, O> {
    pub fn new(
        empty: S,
        input: fn(&T) -> S,
        combine: fn(&S, &S) -> S,
        invert: fn(&S) -> S,
        output: fn(&S) -> O,
    ) -> Self {
        Self {
            state: empty,
            input,
            combine,
            invert,
            output,
        }
    }

    pub fn get(&self) -> O {
        (self.output)(&self.state)
    }
}

impl<T, S, O> Index<T> for MonoidalAggregate<T, S, O> {
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<T>) {
        let inp = (self.input)(op.new);
        self.state = (self.combine)(&self.state, &inp);
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<T>) {
        let inp = (self.input)(op.existing);
        let inv = (self.invert)(&inp);
        self.state = (self.combine)(&self.state, &inv);
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &Update<T>) {
        let old_ = (self.input)(op.existing);
        let new_ = (self.input)(op.new);
        let diff = (self.combine)(&(self.invert)(&old_), &new_);
        (self.combine)(&self.state, &diff);
    }
}
