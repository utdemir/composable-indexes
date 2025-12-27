//! A combinator that filters entries in an index based on a predicate function.
//! Only entries that satisfy the predicate are included in the index.

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal, Update},
};

pub struct Filtered<In, Out, Inner> {
    f: fn(&In) -> Option<Out>,
    inner: Inner,
}

impl<In, Out, Inner> Clone for Filtered<In, Out, Inner>
where
    Inner: Clone,
{
    fn clone(&self) -> Self {
        Filtered {
            f: self.f,
            inner: self.inner.clone(),
        }
    }
}

impl<In, Out, Inner: ShallowClone> ShallowClone for Filtered<In, Out, Inner> {}

impl<In, Out, Inner> Filtered<In, Out, Inner> {
    pub fn new(f: fn(&In) -> Option<Out>, inner: Inner) -> Self {
        Filtered { f, inner }
    }
}

impl<In, Out, Inner> Index<In> for Filtered<In, Out, Inner>
where
    Inner: Index<Out>,
{
    #[inline]
    fn insert(&mut self, seal: Seal, op: &Insert<In>) {
        if let Some(transformed) = (self.f)(op.new) {
            self.inner.insert(
                seal,
                &Insert {
                    key: op.key,
                    new: &transformed,
                },
            );
        }
    }

    #[inline]
    fn update(&mut self, seal: Seal, op: &Update<In>) {
        let new_opt = (self.f)(op.new);
        let existing_opt = (self.f)(op.existing);

        match (existing_opt, new_opt) {
            (Some(existing), Some(new)) => {
                self.inner.update(
                    seal,
                    &Update {
                        key: op.key,
                        new: &new,
                        existing: &existing,
                    },
                );
            }
            (Some(existing), None) => {
                self.inner.remove(
                    seal,
                    &Remove {
                        key: op.key,
                        existing: &existing,
                    },
                );
            }
            (None, Some(new)) => {
                self.inner.insert(
                    seal,
                    &Insert {
                        key: op.key,
                        new: &new,
                    },
                );
            }
            (None, None) => {}
        }
    }

    #[inline]
    fn remove(&mut self, seal: Seal, op: &Remove<In>) {
        if let Some(existing) = (self.f)(op.existing) {
            self.inner.remove(
                seal,
                &Remove {
                    key: op.key,
                    existing: &existing,
                },
            );
        }
    }
}

impl<In, Out, Inner> Filtered<In, Out, Inner> {
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl<In, Out, Inner> core::ops::Deref for Filtered<In, Out, Inner> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation;
    use crate::testutils::prop_assert_reference;

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || {
                Filtered::new(
                    |b: &bool| if *b { Some(true) } else { None },
                    aggregation::Count::<u32>::new(),
                )
            },
            |db| db.query(|ix| ix.inner().get()),
            |xs| xs.iter().filter(|&&b| b).count() as u32,
            None,
        );
    }
}
