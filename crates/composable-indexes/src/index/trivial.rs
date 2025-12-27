//! A basic index implementation that maintains no additional data structures.
//! Useful as a no-op index when indexing is not needed.

use crate::{
    ShallowClone,
    core::{Index, Insert, Remove, Seal},
};

#[derive(Clone)]
pub struct Trivial;

impl ShallowClone for Trivial {}

impl<In> Index<In> for Trivial {
    #[inline]
    fn insert(&mut self, _seal: Seal, _op: &Insert<In>) {}
    #[inline]
    fn remove(&mut self, _seal: Seal, _op: &Remove<In>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Collection;

    #[test]
    fn test_basic() {
        let mut coll = Collection::<u8, _>::new(Trivial);

        let key = coll.insert(1);

        let removed_key = coll.insert(2);
        coll.delete_by_key(removed_key);

        coll.insert(3);

        assert_eq!(coll.get_by_key(key), Some(&1));
        assert_eq!(coll.get_by_key(removed_key), None);
    }
}
