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
//! index::premap(|p: &Person| (p.first_name.clone(), p.last_name.clone()), index::btree());
//! ```

use crate::core::{Index, Insert, Remove, Update};

pub fn premap<In, InnerIn, Ix>(f: fn(&In) -> InnerIn, inner: Ix) -> PremapIndex<In, InnerIn, Ix>
where
    Ix: Index<InnerIn>,
{
    PremapIndex { f, inner }
}

#[derive(Clone)]
pub struct PremapIndex<In, InnerIn, Inner> {
    f: fn(&In) -> InnerIn,
    inner: Inner,
}

impl<Inner, In, InnerIn> Index<In> for PremapIndex<In, InnerIn, Inner>
where
    Inner: Index<InnerIn>,
{
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
}

impl<In, InnerIn, Inner> PremapIndex<In, InnerIn, Inner> {
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl<In, InnerIn, Inner> core::ops::Deref for PremapIndex<In, InnerIn, Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
