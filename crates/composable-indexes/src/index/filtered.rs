//! A combinator that filters entries in an index based on a predicate function.
//! Only entries that satisfy the predicate are included in the index.

use composable_indexes_core::{Index, Insert, QueryEnv, Remove, Update};

pub fn filtered<In, Out, Path: Clone, F: Fn(&In) -> Option<Out>, Inner: Index<Out, Path>>(
    f: F,
    inner: Inner,
) -> FilteredIndex<F, Inner> {
    FilteredIndex { f, inner }
}

pub struct FilteredIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<F, Inner, In, Out, Path: Clone> Index<In, Path> for FilteredIndex<F, Inner>
where
    F: Fn(&In) -> Option<Out>,
    Inner: Index<Out, Path>,
{
    type Query<'t, Res>
        = Inner::Query<'t, Res>
    where
        Self: 't,
        Res: 't;

    fn insert(&mut self, op: &Insert<In, Path>) {
        if let Some(transformed) = (self.f)(op.new) {
            self.inner.insert(&Insert {
                key: op.key.clone(),
                new: &transformed,
            });
        }
    }

    fn update(&mut self, op: &Update<In, Path>) {
        let new_opt = (self.f)(op.new);
        let existing_opt = (self.f)(op.existing);

        match (existing_opt, new_opt) {
            (Some(existing), Some(new)) => {
                self.inner.update(&Update {
                    key: op.key.clone(),
                    new: &new,
                    existing: &existing,
                });
            }
            (Some(existing), None) => {
                self.inner.remove(&Remove {
                    key: op.key.clone(),
                    existing: &existing,
                });
            }
            (None, Some(new)) => {
                self.inner.insert(&Insert {
                    key: op.key.clone(),
                    new: &new,
                });
            }
            (None, None) => {}
        }
    }

    fn remove(&mut self, op: &Remove<In, Path>) {
        if let Some(existing) = (self.f)(op.existing) {
            self.inner.remove(&Remove {
                key: op.key.clone(),
                existing: &existing,
            });
        }
    }

    fn query<'t, Res: 't>(&'t self, env: QueryEnv<'t, Res>) -> Self::Query<'t, Res> {
        self.inner.query(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregation;
    use composable_indexes_testutils::prop_assert_reference;

    #[test]
    fn test_reference() {
        prop_assert_reference::<bool, _, _, _, _, _>(
            || {
                filtered::<_, _, _, _, _>(
                    |b: &bool| if *b { Some(true) } else { None },
                    aggregation::count(),
                )
            },
            |q| *q,
            |xs| xs.iter().filter(|&&b| b).count() as u32,
            None,
        );
    }
}
