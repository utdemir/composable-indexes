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
//! Collection::<Person, _>::new(
//!   index::zip!(
//!     // Index by age.
//!     index::premap(|p: &Person| p.age, index::btree()),
//!     // Index by full name
//!     index::premap(|p: &Person| (p.first_name.clone(), p.last_name.clone()), index::hashtable()),
//!   )
//! );
//! ```

use composable_indexes_core::{Index, Insert, QueryEnv, Remove, Update};

pub fn premap<In, Path: Clone, InnerIn, F, Ix>(f: F, inner: Ix) -> PremapIndex<F, Ix>
where
    F: Fn(&In) -> InnerIn,
    Ix: Index<InnerIn, Path>,
{
    PremapIndex { f, inner }
}

pub struct PremapIndex<F, Inner> {
    f: F,
    inner: Inner,
}

impl<F, Inner, Path: Clone, In, InnerIn> Index<In, Path> for PremapIndex<F, Inner>
where
    F: Fn(&In) -> InnerIn,
    Inner: Index<InnerIn, Path>,
{
    type Query<'t, Out>
        = Inner::Query<'t, Out>
    where
        Self: 't,
        Out: 't;

    fn insert(&mut self, op: &Insert<In, Path>) {
        self.inner.insert(&Insert {
            key: op.key.clone(),
            new: &(self.f)(op.new),
        });
    }

    fn update(&mut self, op: &Update<In, Path>) {
        self.inner.update(&Update {
            key: op.key.clone(),
            new: &(self.f)(op.new),
            existing: &(self.f)(op.existing),
        });
    }

    fn remove(&mut self, op: &Remove<In, Path>) {
        self.inner.remove(&Remove {
            key: op.key.clone(),
            existing: &(self.f)(op.existing),
        });
    }

    fn query<'t, Out: 't>(&'t self, env: QueryEnv<'t, Out>) -> Self::Query<'t, Out> {
        self.inner.query(env)
    }
}
