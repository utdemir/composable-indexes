use proptest_derive::Arbitrary;
use std::collections::HashSet;

use composable_indexes::indexes;
use composable_indexes_props::prop_assert_reference;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary)]
enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
}

#[test]
fn test_aggrs() {
    prop_assert_reference(
        || indexes::btree::<Month>(),
        |q| {
            (
                q.max_one().map(|(_k, v)| v).cloned(),
                q.min_one().map(|(_k, v)| v).cloned(),
            )
        },
        |xs| (xs.iter().max().cloned(), xs.iter().min().cloned()),
        None,
    );
}

#[test]
fn test_lookup() {
    prop_assert_reference(
        || indexes::premap(|i: &(Month, u32)| i.1, indexes::btree()),
        |q| {
            q.get_all(&1)
                .iter()
                .map(|i| i.0.clone())
                .collect::<HashSet<Month>>()
        },
        |xs| {
            xs.iter()
                .filter(|i| i.1 == 1)
                .map(|i| i.0.clone())
                .collect::<HashSet<_>>()
        },
        None,
    );
}
