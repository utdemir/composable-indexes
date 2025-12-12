use alloc::vec::Vec;
use proptest::prelude::*;

use super::test_ops::TestOps;
use crate::core::{Collection, Index};

pub fn prop_assert_reference<
    In: Clone + Arbitrary + 'static,
    Ix: Index<In>,
    T: core::fmt::Debug + PartialEq + 'static,
>(
    mk_index: impl Fn() -> Ix,
    query: impl Fn(Collection<In, Ix>) -> T,
    reference_impl: impl Fn(Vec<In>) -> T,
    config: Option<proptest::test_runner::Config>,
) {
    let mut runner = proptest::test_runner::TestRunner::new(config.unwrap_or_default());

    runner
        .run(&any::<TestOps<In>>(), |ops| {
            let ref_xs = ops.end_state().values().cloned().collect::<Vec<_>>();
            let actual = reference_impl(ref_xs);

            let mut db = Collection::new(mk_index());
            ops.apply(&mut db);
            let expected = query(db);

            pretty_assertions::assert_eq!(&actual, &expected);

            Ok(())
        })
        .unwrap();
}
