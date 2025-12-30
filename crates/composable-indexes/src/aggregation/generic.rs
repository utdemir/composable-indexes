use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal, Update},
};

/// Implements an `Index` given state operations.
///
/// Example:
///
/// ```rust
/// use composable_indexes::{Collection, aggregation};
/// let sum_aggregate = aggregation::GenericAggregate::new(
///     // initial state
///     0,
///     // query function
///     |state: &i64| *state,
///     // insert function
///     |state: &mut i64, value: &i64| *state += *value,
///     // remove function
///     |state: &mut i64, value: &i64| *state -= *value,
/// );
/// let mut collection = Collection::new(sum_aggregate);
/// collection.insert(2);
/// collection.insert(3);
/// assert_eq!(collection.query(|ix| ix.get()), 5);
/// ```
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

/// Given an algebraic group, implement an [`Index`].
///
/// Example:
///
/// ```rust
/// use composable_indexes::{Collection, aggregation};
///
/// let number_of_a = aggregation::MonoidalAggregate::new(
//      // empty value
///     0,
///     // input function
///     |x: &String| x.chars().filter(|&c| c == 'a').count() as i64,
///     // combine function
///     |a: &i64, b: &i64| a + b,
///     // invert function
///     |a: &i64| -a,
///     // output function
///     |a: &i64| *a,
/// );
///
/// let mut collection = Collection::new(number_of_a);
/// collection.insert("apple".to_string());
/// collection.insert("banana".to_string());
/// assert_eq!(collection.query(|ix| ix.get()), 4);
/// ```
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
        self.state = (self.combine)(&self.state, &diff);
    }
}

impl<T, S: Clone, O> Clone for MonoidalAggregate<T, S, O> {
    fn clone(&self) -> Self {
        MonoidalAggregate {
            state: self.state.clone(),
            input: self.input,
            combine: self.combine,
            invert: self.invert,
            output: self.output,
        }
    }
}

impl<T, S: Clone, O> ShallowClone for MonoidalAggregate<T, S, O> {}
