use composable_indexes::{Collection, Index};
use proptest::prelude::*;

use crate::TestOps;

pub fn prop_assert_reference<
    In: Clone + Arbitrary + 'static,
    Res: std::fmt::Debug + Clone + PartialEq + Eq,
    Ix: for<'a> Index<'a, In>,
    MkIx: Fn() -> Ix,
    Query: for<'a> Fn(&<Ix as Index<'a, In>>::Query<In>) -> Res,
    ReferenceImpl: Fn(&[In]) -> Res,
>(
    mk_index: MkIx,
    query: Query,
    reference_impl: ReferenceImpl,
    config: Option<proptest::test_runner::Config>,
) {
    let mut runner = proptest::test_runner::TestRunner::new(config.unwrap_or_default());

    runner
        .run(&any::<TestOps<In>>(), |ops| {
            let expected = {
                let xs = ops.end_state().values().cloned().collect::<Vec<_>>();
                let res = reference_impl(&xs);
                res
            };

            let actual = {
                let mut db = Collection::new(mk_index());
                ops.apply(&mut db);
                let q = db.query();
                let res = query(&q);

                res.clone()
            };

            assert_eq!(actual, expected);
            Ok(())
        })
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::prop_assert_reference;
    use composable_indexes::indexes::btree;

    #[test]
    fn test_unit() {
        prop_assert_reference(
            || btree::<u32>(),
            |q| q.max_one().map(|(_k, v)| v).cloned(),
            |xs| xs.iter().max().cloned(),
            None,
        );
    }

    #[test]
    #[should_panic]
    fn test_error() {
        prop_assert_reference(
            || btree::<u32>(),
            |q| std::cmp::min(q.max_one().map(|(_k, v)| v).cloned(), Some(20)),
            |xs| xs.iter().max().cloned(),
            None,
        );
    }
}
