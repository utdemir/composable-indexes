use crate::core::{Index, Insert, QueryEnv, Remove, Update};

pub fn filtered<F, Inner>(predicate: F, inner: Inner) -> FilteredIndex<F, Inner> {
    FilteredIndex { predicate, inner }
}

pub struct FilteredIndex<F, Inner> {
    pub predicate: F,
    pub inner: Inner,
}

impl<'t, F, Inner, In> Index<'t, In> for FilteredIndex<F, Inner>
where
    F: Fn(&In) -> bool + 't,
    Inner: Index<'t, In> + 't,
{
    type Query<Out: 't> = Inner::Query<Out>;

    fn insert(&mut self, op: &Insert<In>) {
        if (self.predicate)(op.new) {
            self.inner.insert(op);
        }
    }

    fn update(&mut self, op: &Update<In>) {
        if (self.predicate)(op.new) {
            self.inner.update(op);
        }
    }

    fn remove(&mut self, op: &Remove<In>) {
        if (self.predicate)(op.existing) {
            self.inner.remove(op);
        }
    }

    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
        self.inner.query(env)
    }
}
