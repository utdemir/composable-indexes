//! A combinator that filters entries in an index based on a predicate function.
//! Only entries that satisfy the predicate are included in the index.

use composable_indexes_core::{Index, Insert, Remove, Update};

pub fn filtered<In, Out, Inner: Index<Out>>(
    f: fn(&In) -> Option<Out>,
    inner: Inner,
) -> FilteredIndex<In, Out, Inner> {
    FilteredIndex { f, inner }
}

pub struct FilteredIndex<In, Out, Inner> {
    f: fn(&In) -> Option<Out>,
    inner: Inner,
}

impl<In, Out, Inner> Index<In> for FilteredIndex<In, Out, Inner>
where
    Inner: Index<Out>,
{
    fn insert(&mut self, op: &Insert<In>) {
        if let Some(transformed) = (self.f)(op.new) {
            self.inner.insert(&Insert {
                key: op.key,
                new: &transformed,
            });
        }
    }

    fn update(&mut self, op: &Update<In>) {
        let new_opt = (self.f)(op.new);
        let existing_opt = (self.f)(op.existing);

        match (existing_opt, new_opt) {
            (Some(existing), Some(new)) => {
                self.inner.update(&Update {
                    key: op.key,
                    new: &new,
                    existing: &existing,
                });
            }
            (Some(existing), None) => {
                self.inner.remove(&Remove {
                    key: op.key,
                    existing: &existing,
                });
            }
            (None, Some(new)) => {
                self.inner.insert(&Insert {
                    key: op.key,
                    new: &new,
                });
            }
            (None, None) => {}
        }
    }

    fn remove(&mut self, op: &Remove<In>) {
        if let Some(existing) = (self.f)(op.existing) {
            self.inner.remove(&Remove {
                key: op.key,
                existing: &existing,
            });
        }
    }
}

impl<In, Out, Inner> FilteredIndex<In, Out, Inner> {
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation;
    use composable_indexes_testutils::prop_assert_reference;

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || {
                filtered(
                    |b: &bool| if *b { Some(true) } else { None },
                    aggregation::count::<u32>(),
                )
            },
            |db| db.query(|ix| ix.inner().get()),
            |xs| xs.iter().filter(|&&b| b).count() as u32,
            None,
        );
    }
}
