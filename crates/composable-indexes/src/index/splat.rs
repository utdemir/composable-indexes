use composable_indexes_core::{Index, Insert, QueryEnv, Remove};

pub fn splat<In, Out, Path: Clone, F: Fn(&In) -> &[Out], Inner: Index<Out, (Path, usize)>>(
    f: F,
    inner: Inner,
) -> SplatIndex<F, Inner> {
    SplatIndex { f, inner }
}

pub struct SplatIndex<F, Inner> {
    pub f: F,
    pub inner: Inner,
}

impl<F, Inner, In, Out: Clone, Path: Clone> Index<In, Path> for SplatIndex<F, Inner>
where
    F: Fn(&In) -> &[Out],
    Inner: Index<Out, (Path, usize)>,
{
    type Query<'t, Res>
        = Inner::Query<'t, Res>
    where
        Self: 't,
        Res: 't;

    fn insert(&mut self, op: &Insert<In, Path>) {
        for (i, p) in (self.f)(op.new).iter().enumerate() {
            self.inner.insert(&Insert {
                key: op.key.push(i),
                new: p,
            });
        }
    }

    fn remove(&mut self, op: &Remove<In, Path>) {
        for (i, p) in (self.f)(op.existing).iter().enumerate() {
            self.inner.remove(&Remove {
                key: op.key.push(i),
                existing: p,
            });
        }
    }

    fn query<'t, Res: 't>(&'t self, env: QueryEnv<'t, Res>) -> Self::Query<'t, Res> {
        self.inner.query(env)
    }
}

#[cfg(test)]
mod tests {
    use composable_indexes_testutils::prop_assert_reference;
    use proptest_derive::Arbitrary;

    use super::*;
    use crate::{aggregation, index::premap};

    #[derive(Debug, Clone, Arbitrary)]
    struct Foo {
        nums: Vec<u8>,
    }

    #[test]
    fn test_reference() {
        prop_assert_reference(
            || splat(|p: &Foo| &p.nums, premap(|i| *i as u64, aggregation::sum())),
            |q| *q,
            |xs| {
                xs.iter()
                    .flat_map(|x| x.nums.iter().map(|y| *y as u64))
                    .sum()
            },
            None,
        );
    }
}
