//! A combinator that transforms the input type of an index using a mapping function.
//! This allows reusing existing indexes with different data types by pre-processing the input.
//!
//! # Example
//!
//! ```rust
//! use composable_indexes::{Collection, index};
//!
//! struct Person { first_name: String, last_name: String, age: u32 }
//!
//! // Index by age.
//! index::premap(|p: &Person| p.age, index::btree());
//!
//! // Index by full name
//! index::premap(|p: &Person| (p.first_name.clone(), p.last_name.clone()), index::hashtable());
//! ```

use composable_indexes_core::{Index, Insert, QueryEnv, Remove, Update};

pub fn premap<In, InnerIn, F, Ix>(f: F, inner: Ix) -> PremapIndex<F, Ix>
where
    F: Fn(&In) -> InnerIn,
    Ix: Index<InnerIn>,
{
    PremapIndex { f, inner }
}

pub struct PremapIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<F, Inner, In, InnerIn> Index<In> for PremapIndex<F, Inner>
where
    F: Fn(&In) -> InnerIn,
    Inner: Index<InnerIn>,
{
    type Query<'t, Out>
        = Inner::Query<'t, Out>
    where
        Self: 't,
        Out: 't;

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

    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        self.inner.query(env)
    }
}
