//! A basic index implementation that maintains no additional data structures.
//! Useful as a no-op index when indexing is not needed.

use composable_indexes_core::{Index, Insert, Remove};

pub fn trivial() -> TrivialIndex {
    TrivialIndex
}

pub struct TrivialIndex;

impl<In> Index<In> for TrivialIndex {
    fn insert(&mut self, _op: &Insert<In>) {}
    fn remove(&mut self, _op: &Remove<In>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use composable_indexes_core::Collection;

    #[test]
    fn test_basic() {
        let mut coll = Collection::<u8, _>::new(trivial());

        let key = coll.insert(1);

        let removed_key = coll.insert(2);
        coll.delete(&removed_key);

        coll.insert(3);

        assert_eq!(coll.get(key), Some(&1));
        assert_eq!(coll.get(removed_key), None);
    }
}
