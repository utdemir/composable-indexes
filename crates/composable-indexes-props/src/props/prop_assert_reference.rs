use composable_indexes_core::{Collection, Index};
use proptest::prelude::*;

use crate::TestOps;

pub fn prop_assert_reference<
    In: Clone + Arbitrary + 'static,
    Res: std::fmt::Debug + Clone + PartialEq,
    Ix: Index<In>,
    MkIx: Fn() -> Ix,
    Query: for<'a> Fn(&<Ix as Index<In>>::Query<'a, In>) -> Res,
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
