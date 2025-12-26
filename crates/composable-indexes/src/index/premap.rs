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
//! index::PremapOwnedIndex::new(|p: &Person| p.age, index::btree::<u32>());
//!
//! // Index by full name (owned value)
//! index::PremapOwnedIndex::new(|p: &Person| (p.first_name.clone(), p.last_name.clone()), index::btree::<(String, String)>());
//! ```

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal, Update},
};

/// Generic premap index that takes a function as a type parameter
pub struct GenericPremapIndex<In, InnerIn, F, Inner> {
    f: F,
    inner: Inner,
    _phantom: core::marker::PhantomData<(In, InnerIn)>,
}

impl<In, InnerIn, F, Inner> Clone for GenericPremapIndex<In, InnerIn, F, Inner>
where
    F: Copy,
    Inner: Clone,
{
    fn clone(&self) -> Self {
        GenericPremapIndex {
            f: self.f,
            inner: self.inner.clone(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<In, InnerIn, F, Inner> ShallowClone for GenericPremapIndex<In, InnerIn, F, Inner>
where
    Inner: ShallowClone,
    F: Copy,
{
}

/// Type alias for premap index with references (function returns &InnerIn)
pub type PremapIndex<In, InnerIn, Inner> =
    GenericPremapIndex<In, InnerIn, fn(&In) -> &InnerIn, Inner>;

/// Type alias for premap index with owned values (function returns InnerIn)
pub type PremapOwnedIndex<In, InnerIn, Inner> =
    GenericPremapIndex<In, InnerIn, fn(&In) -> InnerIn, Inner>;

impl<In, InnerIn, Inner> PremapIndex<In, InnerIn, Inner> {
    pub fn new(f: fn(&In) -> &InnerIn, inner: Inner) -> Self {
        PremapIndex {
            f,
            inner,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<In, InnerIn, Inner> PremapOwnedIndex<In, InnerIn, Inner> {
    pub fn new(f: fn(&In) -> InnerIn, inner: Inner) -> Self {
        PremapOwnedIndex {
            f,
            inner,
            _phantom: core::marker::PhantomData,
        }
    }
}

// Index implementation for reference-based premap
impl<In, InnerIn, Inner> Index<In> for PremapIndex<In, InnerIn, Inner>
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
impl<In, InnerIn, Inner> Index<In> for PremapOwnedIndex<In, InnerIn, Inner>
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

impl<In, InnerIn, F, Inner> GenericPremapIndex<In, InnerIn, F, Inner> {
    #[inline]
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl<In, InnerIn, F, Inner> core::ops::Deref for GenericPremapIndex<In, InnerIn, F, Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
