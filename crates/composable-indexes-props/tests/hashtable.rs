use std::{collections::HashSet, hash::Hash};

use composable_indexes::indexes;
use composable_indexes_props::prop_assert_reference;
use proptest_derive::Arbitrary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Arbitrary)]
enum Month {
    Jan,
    Feb,
    Mar,
    Apr,
}

#[test]
fn test_lookup() {
    prop_assert_reference(
        || indexes::hashtable(),
        |q| {
            q.get_all(&Month::Mar)
                .iter()
                .map(|&month| month.clone())
                .collect::<HashSet<_>>()
        },
        |xs| {
            xs.iter()
                .filter(|&&month| month == Month::Mar)
                .map(|&month| month.clone())
                .collect::<HashSet<_>>()
        },
        None,
    );
}
