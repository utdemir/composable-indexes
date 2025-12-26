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
//! // Index by age (owned value).
//! index::PremapOwned::new(|p: &Person| p.age, index::BTreeIndex::<u32>::new());
//!
//! // Index by full name (owned value)
//! index::PremapOwned::new(|p: &Person| (p.first_name.clone(), p.last_name.clone()), index::BTreeIndex::<(String, String)>::new());
//! ```

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal, Update},
};

/// Generic premap index that takes a function as a type parameter
pub struct GenericPremap<In, InnerIn, F, Inner> {
    f: F,
    inner: Inner,
    _phantom: core::marker::PhantomData<(In, InnerIn)>,
}

impl<In, InnerIn, F, Inner> Clone for GenericPremap<In, InnerIn, F, Inner>
where
    F: Copy,
    Inner: Clone,
{
    fn clone(&self) -> Self {
        GenericPremap {
            f: self.f,
            inner: self.inner.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<In, InnerIn, F, Inner> ShallowClone for GenericPremap<In, InnerIn, F, Inner>
where
    Inner: ShallowClone,
    F: Copy,
{
}

/// Type alias for premap index with references (function returns &InnerIn)
pub type Premap<In, InnerIn, Inner> = GenericPremap<In, InnerIn, fn(&In) -> &InnerIn, Inner>;

/// Type alias for premap index with owned values (function returns InnerIn)
pub type PremapOwned<In, InnerIn, Inner> = GenericPremap<In, InnerIn, fn(&In) -> InnerIn, Inner>;

impl<In, InnerIn, Inner> Premap<In, InnerIn, Inner> {
    pub fn new(f: fn(&In) -> &InnerIn, inner: Inner) -> Self {
        Premap {
            f,
            inner,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<In, InnerIn, Inner> PremapOwned<In, InnerIn, Inner> {
    pub fn new(f: fn(&In) -> InnerIn, inner: Inner) -> Self {
        PremapOwned {
            f,
            inner,
            _phantom: core::marker::PhantomData,
        }
    }
}

// Index implementation for reference-based premap
impl<In, InnerIn, Inner> Index<In> for Premap<In, InnerIn, Inner>
where
    Inner: Index<InnerIn>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<In>) {
        self.inner.insert(
            seal,
            &Insert {
                key: op.key,
                new: (self.f)(op.new),
            },
        );
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<In>) {
        self.inner.update(
            seal,
            &Update {
                key: op.key,
                new: (self.f)(op.new),
                existing: (self.f)(op.existing),
            },
        );
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<In>) {
        self.inner.remove(
            seal,
            &Remove {
                key: op.key,
                existing: (self.f)(op.existing),
            },
        );
    }
}

// Index implementation for owned-based premap
impl<In, InnerIn, Inner> Index<In> for PremapOwned<In, InnerIn, Inner>
where
    Inner: Index<InnerIn>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<In>) {
        self.inner.insert(
            seal,
            &Insert {
                key: op.key,
                new: &(self.f)(op.new),
            },
        );
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<In>) {
        self.inner.update(
            seal,
            &Update {
                key: op.key,
                new: &(self.f)(op.new),
                existing: &(self.f)(op.existing),
            },
        );
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<In>) {
        self.inner.remove(
            seal,
            &Remove {
                key: op.key,
                existing: &(self.f)(op.existing),
            },
        );
    }
}

impl<In, InnerIn, F, Inner> GenericPremap<In, InnerIn, F, Inner> {
    #[inline]
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl<In, InnerIn, F, Inner> core::ops::Deref for GenericPremap<In, InnerIn, F, Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
