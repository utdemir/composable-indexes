use num_traits::Num;

use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

/// An index that provides the sum of indexed values.
#[derive(Clone)]
pub struct Sum<T> {
    sum: T,
}

impl<T: Num + Copy> Default for Sum<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Num + Copy> Sum<T> {
    pub fn new() -> Self {
        Sum { sum: T::zero() }
    }
}

impl<T> Index<T> for Sum<T>
where
    T: Num + Copy + 'static,
{
    #[inline]
    fn insert(&mut self, _seal: Seal, op: &Insert<T>) {
        self.sum = self.sum + *op.new;
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, op: &Remove<T>) {
        self.sum = self.sum - *op.existing;
    }

    #[inline]
    fn update(&mut self, _seal: Seal, op: &Update<T>) {
        self.sum = self.sum - *op.existing + *op.new;
    }
}

impl<T: Copy> Sum<T> {
    #[inline]
    pub fn get(&self) -> T {
        self.sum
    }
}

impl<T: Clone> ShallowClone for Sum<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutils::prop_assert_reference;
    use core::num::Wrapping;

    #[test]
    fn test_sum() {
        prop_assert_reference(
            Sum::<Wrapping<i16>>::new,
            |db| db.query(|ix| ix.get().0),
            |xs| xs.iter().map(|x| Wrapping(x.0)).sum::<Wrapping<i16>>().0,
            None,
        );
    }
}
