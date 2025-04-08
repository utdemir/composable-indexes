use composable_indexes_core::{Index, Insert, QueryEnv, Remove, Update};

pub fn premap<'t, In, F, InnerIn, Ix>(f: F, inner: Ix) -> PremapIndex<F, Ix>
where
    F: Fn(&In) -> InnerIn + 't,
    In: 't,
    Ix: Index<'t, InnerIn>,
{
    PremapIndex { f, inner }
}

pub struct PremapIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<'t, F, Inner, In, InnerIn> Index<'t, In> for PremapIndex<F, Inner>
where
    F: Fn(&In) -> InnerIn + 't,
    Inner: Index<'t, InnerIn> + 't,
{
    type Query<Out: 't> = Inner::Query<Out>;

    fn insert(&mut self, op: &Insert<In>) {
        self.inner.insert(&Insert {
            key: op.key,
            new: &(self.f)(op.new),
        });
    }

    fn update(&mut self, op: &Update<In>) {
        self.inner.update(&Update {
            key: op.key,
            new: &(self.f)(op.new),
            existing: &(self.f)(op.existing),
        });
    }

    fn remove(&mut self, op: &Remove<In>) {
        self.inner.remove(&Remove {
            key: op.key,
            existing: &(self.f)(op.existing),
        });
    }

    fn query<Out>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<Out> {
        self.inner.query(env)
    }
}
