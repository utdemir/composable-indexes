use crate::core::{Index, Insert, QueryEnv, Remove, Update};

pub fn filtered<'t, In, Out, F: Fn(&In) -> Option<&Out>, Inner: Index<'t, In>>(
    f: F,
    inner: Inner,
) -> FilteredIndex<F, Inner> {
    FilteredIndex { f, inner }
}

pub struct FilteredIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<'t, F, Inner, In, Out> Index<'t, In> for FilteredIndex<F, Inner>
where
    F: Fn(&In) -> Option<Out> + 't,
    Inner: Index<'t, Out> + 't,
{
    type Query<Res: 't> = Inner::Query<Res>;

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

    fn query<Res>(&'t self, env: QueryEnv<'t, Res>) -> Self::Query<Res> {
        self.inner.query(env)
    }
}
