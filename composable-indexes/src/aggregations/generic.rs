use crate::Index;

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

impl<'t, In: 't, Query: 't, State: 't> Index<'t, In> for AggregateIndex<In, Query, State>
where
    State: Clone,
    Query: Clone,
{
    type Query<Out: 't> = Query;

    fn insert(&mut self, op: &crate::Insert<In>) {
        (self.insert)(&mut self.current_state, op.new);
    }

    fn update(&mut self, op: &crate::Update<In>) {
        (self.remove)(&mut self.current_state, op.existing);
        (self.insert)(&mut self.current_state, op.new);
    }

    fn remove(&mut self, op: &crate::Remove<In>) {
        (self.remove)(&mut self.current_state, op.existing);
    }

    fn query<Out>(&self, _env: crate::QueryEnv<'t, Out>) -> Self::Query<Out> {
        (self.query)(&self.current_state)
    }
}
