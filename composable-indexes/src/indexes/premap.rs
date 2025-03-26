use crate::core::{Index, Insert, QueryEnv, Remove, Update};

pub fn premap<F, Inner>(f: F, inner: Inner) -> PremapIndex<F, Inner> {
    PremapIndex { f, inner }
}

pub struct PremapIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<'t, F, Inner, In, InnerIn, Out> Index<'t, In, Out> for PremapIndex<F, Inner>
where
    F: Fn(&In) -> InnerIn,
    Inner: Index<'t, InnerIn, Out>,
{
    type Queries = Inner::Queries;

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

    fn query(&'t self, env: QueryEnv<'t, Out>) -> Self::Queries {
        self.inner.query(env)
    }
}
