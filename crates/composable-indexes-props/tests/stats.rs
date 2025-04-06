use std::num::Wrapping;

use composable_indexes::aggregations;
use composable_indexes_props::prop_assert_reference;

#[test]
fn test_sum() {
    prop_assert_reference(
        || aggregations::sum::<Wrapping<i16>>(),
        |q| *q,
        |xs| xs.iter().sum(),
        None,
    );
}
