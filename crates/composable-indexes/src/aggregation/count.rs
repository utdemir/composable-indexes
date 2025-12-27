use num_traits::Num;

use crate::{
    Index, ShallowClone,
    core::{Insert, Remove, Seal, Update},
};

#[derive(Clone)]
pub struct Count<T = u64> {
    count: T,
}

impl<T: Num + Copy> Default for Count<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Num + Copy> Count<T> {
    pub fn new() -> Self {
        Count { count: T::zero() }
    }
}

impl<T, _K> Index<_K> for Count<T>
where
    T: Num + Copy + 'static,
{
    #[inline]
    fn insert(&mut self, _seal: Seal, _op: &Insert<_K>) {
        self.count = self.count + T::one();
    }

    #[inline]
    fn remove(&mut self, _seal: Seal, _op: &Remove<_K>) {
        self.count = self.count - T::one();
    }

    #[inline]
    fn update(&mut self, _seal: Seal, _op: &Update<_K>) {
        // No change in count on update
    }
}

impl<T: Copy> Count<T> {
    #[inline]
    pub fn get(&self) -> T {
        self.count
    }
}

impl<T: Clone> ShallowClone for Count<T> {}
