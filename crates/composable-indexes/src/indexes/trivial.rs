use composable_indexes_core::{Index, Insert, Key, QueryEnv, Remove, Update};

pub fn trivial() -> TrivialIndex {
    TrivialIndex
}

pub struct TrivialQueries<'t, Out> {
    env: QueryEnv<'t, Out>,
}

impl<Out> TrivialQueries<'_, Out> {
    pub fn get(&self, key: &Key) -> Option<&Out> {
        self.env.data.get(key)
    }
}

pub struct TrivialIndex;

impl<'t, In: 't> Index<'t, In> for TrivialIndex {
    type Query<Out: 't> = TrivialQueries<'t, Out>;
    fn insert(&mut self, _op: &Insert<In>) {}
    fn update(&mut self, _op: &Update<In>) {}
    fn remove(&mut self, _op: &Remove<In>) {}
    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
        TrivialQueries { env }
    }
}
